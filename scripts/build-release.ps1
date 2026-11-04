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
$localPointer = Join-Path $root '.tooling\oauth-config-path.txt'
if ([string]::IsNullOrWhiteSpace($OAuthJsonPath) -and [string]::IsNullOrWhiteSpace($env:SHEETS_MONITOR_OAUTH_CLIENT_JSON) -and (Test-Path -LiteralPath $localPointer -PathType Leaf)) {
    $OAuthJsonPath = (Get-Content -Raw -LiteralPath $localPointer).Trim()
}
if (-not [string]::IsNullOrWhiteSpace($OAuthJsonPath)) {
    $resolvedJson = [IO.Path]::GetFullPath($OAuthJsonPath)
    if (-not (Test-Path -LiteralPath $resolvedJson -PathType Leaf)) {
        throw 'Arquivo JSON OAuth Desktop informado não foi encontrado.'
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

    $tauriConf = Get-Content -Raw -LiteralPath (Join-Path $root 'src-tauri\tauri.conf.json') | ConvertFrom-Json
    $version = [string]$tauriConf.version
    $candidateRels = @(
        "src-tauri/target/release/bundle/nsis/Sheets Monitor_${version}_x64-setup.exe",
        "src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Sheets Monitor_${version}_x64-setup.exe"
    )
    $installerRel = $candidateRels | Where-Object { Test-Path -LiteralPath (Join-Path $root $_) -PathType Leaf } | Select-Object -First 1
    if (-not $installerRel) {
        throw "Instalador NSIS esperado não encontrado em: $($candidateRels -join ' ou ')"
    }
    $installerPath = Join-Path $root $installerRel
    $binaryDir = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $installerPath))
    $binaryPath = Join-Path $binaryDir 'sheets-monitor.exe'

    $publicAssetName = "Sheets.Monitor_${version}_x64-setup.exe"
    $outDir = Join-Path $root ".tooling\release-artifacts\v$version"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null

    $publicInstallerPath = Join-Path $outDir $publicAssetName
    Copy-Item -LiteralPath $installerPath -Destination $publicInstallerPath -Force

    $installerHash = (Get-FileHash -LiteralPath $publicInstallerPath -Algorithm SHA256).Hash.ToUpperInvariant()
    $binaryHash = if (Test-Path -LiteralPath $binaryPath -PathType Leaf) {
        (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToUpperInvariant()
    } else {
        ''
    }
    $installerSize = (Get-Item -LiteralPath $publicInstallerPath).Length
    $commitSha = (& git rev-parse HEAD 2>$null | Out-String).Trim()
    $sigStatus = (Get-AuthenticodeSignature -LiteralPath $publicInstallerPath).Status.ToString()

    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    $sumsLine = "$installerHash  $publicAssetName`n"
    [IO.File]::WriteAllText((Join-Path $outDir 'SHA256SUMS.txt'), $sumsLine, $utf8NoBom)

    $manifest = [ordered]@{
        version = $version
        source_commit = $commitSha
        status = 'OAUTH_PROVISIONED_STABLE_RELEASE'
        target = 'x86_64-pc-windows-msvc'
        installer_path = $installerRel
        bytes = $installerSize
        sha256 = $installerHash
        binary_sha256 = $binaryHash
        signature_status = $sigStatus
        build_exit_code = 0
        oauth_desktop_type = $true
        oauth_client_embedded = $true
        oauth_desktop_secret_embedded = $true
        oauth_audience = 'internal_user_reported'
        public_verification_required_for_internal = $false
        native_probe_enabled = $false
        public_asset_name = $publicAssetName
    } | ConvertTo-Json -Depth 4
    [IO.File]::WriteAllText((Join-Path $outDir 'artifact-manifest.json'), "$manifest`n", $utf8NoBom)
} finally {
    Pop-Location
}

Write-Host 'PASS: Build de distribuição NSIS e manifesto SHA-256 concluídos com cliente OAuth Desktop provisionado.'
