[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'environment-common.ps1')
Update-ProcessToolPath
$cpp = Get-CppInstallation
if (-not $cpp) { throw 'MSVC completo necessário.' }
Import-CppEnvironment $cpp
$root = Get-EnvironmentRoot
$evidence = Join-Path $root 'docs/evidence/sprint06'
$artifacts = Join-Path $root '.tooling/load-artifacts.jsonl'
$testName = 'monitoring::service::tests::thirty_workbooks_queue_quota_recovery_and_persistence'
New-Item -ItemType Directory -Force $evidence, (Join-Path $root '.tooling') | Out-Null
& cargo test --manifest-path (Join-Path $root 'src-tauri/Cargo.toml') --locked --offline --target x86_64-pc-windows-msvc --lib --no-run --message-format=json 1> $artifacts 2> (Join-Path $evidence 'load-compile.log')
if ($LASTEXITCODE -ne 0) { throw 'Compilação da campanha falhou; consulte load-compile.log.' }
$binaries = @(Get-Content $artifacts | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.executable -and $_.profile.test })
if ($binaries.Count -ne 1) { throw 'Artefato de teste não é único.' }
$binary = $binaries[0].executable
$timer = [Diagnostics.Stopwatch]::StartNew()
$process = Start-Process -FilePath $binary -ArgumentList @($testName, '--exact', '--nocapture') -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $evidence 'load-run.log') -RedirectStandardError (Join-Path $evidence 'load-metrics.log')
$working = 0L
$private = 0L
while (-not $process.HasExited) {
    if ($timer.Elapsed.TotalSeconds -gt 120) {
        $process.Kill()
        throw 'Timeout da campanha sintética isolada; somente o processo lançado foi encerrado.'
    }
    $process.Refresh()
    $working = [Math]::Max($working, $process.PeakWorkingSet64)
    $private = [Math]::Max($private, $process.PrivateMemorySize64)
    Start-Sleep -Milliseconds 100
}
$process.WaitForExit()
$timer.Stop()
$code = $process.ExitCode
$executed = (Get-Content (Join-Path $evidence 'load-run.log') -Raw) -match 'test result: ok\. 1 passed;'
@{
    kind = 'synthetic_scheduler_sqlite_test_process'
    google_live = $false
    soak_test = $false
    files = 30
    test = $testName
    elapsed_ms = $timer.ElapsedMilliseconds
    observed_peak_working_set_bytes = $working
    sampled_max_private_bytes = $private
    memory_sample_ms = 100
    executable_sha256 = (Get-FileHash $binary -Algorithm SHA256).Hash
    exit_code = $code
    passed = ($null -ne $code -and $code -eq 0 -and $executed)
    limits = 'Synthetic clock/transport/account; not full desktop memory or Google throughput. No claim of sustained wall-clock duration.'
} | ConvertTo-Json | Set-Content (Join-Path $evidence 'load-validation.json')
if ($null -eq $code -or $code -ne 0 -or -not $executed) { throw "Campanha falhou ou nenhum teste executado: $code" }
Write-Host 'PASS: campanha sintética; métricas em docs/evidence/sprint06/load-validation.json.'
