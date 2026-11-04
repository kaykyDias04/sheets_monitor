[CmdletBinding()]
param([string]$EvidenceDirectory = 'docs/evidence/sprint06/load')

# Runs only the synthetic Sprint 06 test campaign. No production profile,
# Google access, native toast or Windows startup registration is involved.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$evidence = [IO.Path]::GetFullPath((Join-Path $root $EvidenceDirectory))
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
. (Join-Path $PSScriptRoot 'environment-common.ps1')
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
$manifest = Join-Path $root 'src-tauri/Cargo.toml'
$buildLog = Join-Path $evidence 'compile.jsonl'
$buildErrors = Join-Path $evidence 'compile-stderr.log'
& cargo test --manifest-path $manifest --locked --target x86_64-pc-windows-msvc --lib --no-run --message-format=json 2> $buildErrors | Set-Content -LiteralPath $buildLog -Encoding utf8
$compileExit = $LASTEXITCODE
if ($compileExit -ne 0) { throw "Campaign compilation failed ($compileExit); see $buildErrors" }
$artifacts = @(Get-Content -LiteralPath $buildLog | ForEach-Object {
    $item = $_ | ConvertFrom-Json
    if ($item.reason -eq 'compiler-artifact' -and $item.executable -and $item.profile.test -and $item.target.name -eq 'sheets_monitor_lib') { $item.executable }
})
if ($artifacts.Count -ne 1) { throw 'Expected exactly one library test executable.' }
$start = New-Object Diagnostics.ProcessStartInfo
$start.FileName = $artifacts[0]
$start.Arguments = 'sprint06 --nocapture --test-threads=1'
$start.WorkingDirectory = Join-Path $root 'src-tauri'
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$process = New-Object Diagnostics.Process
$process.StartInfo = $start
$wall = [Diagnostics.Stopwatch]::StartNew()
if (-not $process.Start()) { throw 'Could not start synthetic campaign process.' }
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$samples = New-Object 'Collections.Generic.List[object]'
$lastCpuMs = 0.0
$peakWorkingSet = 0L
$peakPrivateBytes = 0L
$osPeakWorkingSet = 0L
while (-not $process.HasExited) {
    try {
        $process.Refresh()
        $lastCpuMs = $process.TotalProcessorTime.TotalMilliseconds
        $peakWorkingSet = [Math]::Max($peakWorkingSet, $process.WorkingSet64)
        $peakPrivateBytes = [Math]::Max($peakPrivateBytes, $process.PrivateMemorySize64)
        $osPeakWorkingSet = [Math]::Max($osPeakWorkingSet, $process.PeakWorkingSet64)
        $samples.Add([pscustomobject]@{ elapsed_ms = $wall.ElapsedMilliseconds; cpu_ms = $lastCpuMs; working_set_bytes = $process.WorkingSet64; private_bytes = $process.PrivateMemorySize64 })
    } catch [InvalidOperationException] { }
    # Sampling cadence only; assertions and completion use bounded test predicates.
    [Threading.Thread]::Sleep(25)
}
$process.WaitForExit()
$wall.Stop()
$exitCode = $process.ExitCode
$output = $stdout.GetAwaiter().GetResult()
$errors = $stderr.GetAwaiter().GetResult()
[IO.File]::WriteAllText((Join-Path $evidence 'campaign.log'), $output, (New-Object Text.UTF8Encoding($false)))
[IO.File]::WriteAllText((Join-Path $evidence 'campaign-stderr.log'), $errors, (New-Object Text.UTF8Encoding($false)))
$samples | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidence 'process-samples.json') -Encoding utf8
$metrics = @($output -split '\r?\n' | ForEach-Object {
    $marker = 'SPRINT06_METRIC '
    $offset = $_.IndexOf($marker)
    if ($offset -ge 0) { $_.Substring($offset + $marker.Length) | ConvertFrom-Json }
})
$summary = [ordered]@{
    synthetic = $true
    process_id = $process.Id
    executable_sha256 = (Get-FileHash -LiteralPath $artifacts[0] -Algorithm SHA256).Hash
    test_filter = 'sprint06'
    exit_code = $exitCode
    wall_elapsed_ms = $wall.ElapsedMilliseconds
    cpu_ms_last_sample = $lastCpuMs
    cpu_measurement = 'TotalProcessorTime of exact test PID; last live sample, may exclude final <25ms'
    sample_period_ms = 25
    sample_count = $samples.Count
    peak_sampled_working_set_bytes = $peakWorkingSet
    peak_sampled_private_bytes = $peakPrivateBytes
    peak_working_set_os_bytes = $osPeakWorkingSet
    memory_scope = 'Whole synthetic test process including runtime and three sequential campaign sizes; not per-size RSS'
    campaigns = $metrics
    google_requests = 0
    native_toasts = 0
    real_google_load_executed = $false
    soak_24h_executed = $false
    success = ($exitCode -eq 0 -and $metrics.Count -eq 3)
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'metrics.json') -Encoding utf8
$process.Dispose()
if (-not $summary.success) { throw "Synthetic campaign failed ($exitCode); logs preserved in $evidence" }
Write-Host "PASS: synthetic 5/15/30 campaign; metrics in $evidence"
