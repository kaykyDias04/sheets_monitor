[CmdletBinding()]
param(
    [string]$OAuthJsonPath = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'environment-common.ps1')
Update-ProcessToolPath

$installation = Get-CppInstallation
if (-not $installation) {
    throw 'MSVC x64 ausente. Execute scripts/doctor.ps1 e consulte docs/environment.md.'
}
Import-CppEnvironment $installation

$root = Split-Path -Parent $PSScriptRoot
if (-not [string]::IsNullOrWhiteSpace($OAuthJsonPath)) {
    $resolvedJson = [IO.Path]::GetFullPath($OAuthJsonPath)
    if (-not (Test-Path -LiteralPath $resolvedJson -PathType Leaf)) {
        throw "Arquivo JSON OAuth Desktop não encontrado: $resolvedJson"
    }
    $env:SHEETS_MONITOR_OAUTH_CLIENT_JSON = $resolvedJson
}

# Exige provisionamento OAuth Desktop válido antes de permitir gerar instalador de distribuição.
$env:SHEETS_MONITOR_REQUIRE_OAUTH = '1'

Push-Location $root
try {
    & npm.cmd run tauri -- build --bundles nsis
    if ($LASTEXITCODE -ne 0) {
        throw "Falha ao compilar pacote NSIS Tauri (exit code $LASTEXITCODE)."
    }
} finally {
    Pop-Location
}

Write-Host 'PASS: Build de distribuição NSIS concluído com cliente OAuth Desktop provisionado.'

