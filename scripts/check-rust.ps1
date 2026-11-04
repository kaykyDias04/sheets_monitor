[CmdletBinding()]
param(
    [ValidateSet('test','clippy','build','format-check','format')]
    [string]$Task = 'test',
    [string]$EvidenceDirectory = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'environment-common.ps1')
Update-ProcessToolPath
$installation = Get-CppInstallation
if (-not $installation) { throw 'MSVC x64 ausente. Execute scripts/doctor.ps1 e consulte docs/environment.md.' }
Import-CppEnvironment $installation
$cargo = Get-NativeTool 'cargo.exe'
if (-not $cargo) { throw 'Cargo ausente. Consulte docs/environment.md.' }
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root 'src-tauri\Cargo.toml'
$evidence = if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) {
    Join-Path $root 'docs\evidence\sprint06\rust'
} else {
    [IO.Path]::GetFullPath($EvidenceDirectory)
}
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$arguments = switch ($Task) {
    'test' { @('test','--manifest-path',$manifest,'--locked','--target','x86_64-pc-windows-msvc','--all-targets') }
    'clippy' { @('clippy','--manifest-path',$manifest,'--locked','--target','x86_64-pc-windows-msvc','--all-targets','--','-D','warnings') }
    'build' { @('build','--manifest-path',$manifest,'--locked','--target','x86_64-pc-windows-msvc','--features','custom-protocol') }
    'format-check' { @('fmt','--manifest-path',$manifest,'--all','--','--check') }
    'format' { @('fmt','--manifest-path',$manifest,'--all') }
}
$null = Invoke-ToolChecked $cargo $arguments (Join-Path $evidence "rust-$Task.log")
Write-Host "PASS: Rust $Task (MSVC x64)."
