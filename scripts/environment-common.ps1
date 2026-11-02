# Shared local environment inspection. No installation happens in this file.
Set-StrictMode -Version Latest

function Get-EnvironmentRoot {
    Split-Path -Parent $PSScriptRoot
}

function Update-ProcessToolPath {
    $candidates = @($env:Path -split ';') + @(
        [Environment]::GetEnvironmentVariable('Path', 'Machine') -split ';'
    ) + @([Environment]::GetEnvironmentVariable('Path', 'User') -split ';') + @(
        (Join-Path $env:USERPROFILE '.cargo\bin'),
        (Join-Path $env:APPDATA 'npm'),
        (Join-Path $env:ProgramFiles 'nodejs'),
        (Join-Path $env:ProgramFiles 'Git\cmd')
    )
    $unique = [System.Collections.Generic.List[string]]::new()
    foreach ($entry in $candidates) {
        if ([string]::IsNullOrWhiteSpace($entry)) { continue }
        $entry = [Environment]::ExpandEnvironmentVariables($entry.Trim().Trim('"'))
        if (-not ($unique -contains $entry)) { $unique.Add($entry) }
    }
    $env:Path = $unique -join ';'
}

function Add-UserToolPath([string]$Directory) {
    if (-not (Test-Path -LiteralPath $Directory -PathType Container)) { return }
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries = @($userPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $expanded = @($entries | ForEach-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') })
    if ($expanded -notcontains $Directory.TrimEnd('\')) {
        [Environment]::SetEnvironmentVariable('Path', (($entries + $Directory) -join ';'), 'User')
        Write-Host "PATH do usuario: adicionado $Directory"
    }
    Update-ProcessToolPath
}

function Get-NativeTool([string]$Name) {
    $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($command) { return $command.Source }
    return $null
}

function Invoke-ToolChecked([string]$File, [string[]]$Arguments, [string]$LogPath) {
    if ([string]::IsNullOrWhiteSpace($File) -or -not (Test-Path -LiteralPath $File -PathType Leaf)) {
        throw "Executavel local nao encontrado: $File"
    }
    $previousPreference = $ErrorActionPreference
    try {
        # Native stderr also contains successful progress messages (Rust/npm).
        $ErrorActionPreference = 'Continue'
        if ($LogPath) {
            $output = @(& $File @Arguments 2>&1 | ForEach-Object { "$_" } | Tee-Object -FilePath $LogPath)
        } else {
            $output = @(& $File @Arguments 2>&1 | ForEach-Object { "$_" })
        }
        $code = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previousPreference }
    if ($code -ne 0) { throw "$File terminou com codigo $code. $($output -join [Environment]::NewLine)" }
    return ($output -join [Environment]::NewLine).Trim()
}

function Get-CppInstallation([switch]$IncludeIncomplete) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { return $null }
    $arguments = @('-latest','-products','*','-requires','Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-format','json')
    if ($IncludeIncomplete) { $arguments = @('-all','-latest','-products','Microsoft.VisualStudio.Product.BuildTools','-format','json') }
    $json = & $vswhere @arguments
    if ($LASTEXITCODE -ne 0) { return $null }
    $installations = @($json | ConvertFrom-Json)
    if ($installations.Count -eq 0) { return $null }
    $installation = $installations[0]
    if (-not $IncludeIncomplete -and (-not $installation.isComplete -or -not $installation.isLaunchable)) { return $null }
    if ($installation.isPrerelease) { return $null }
    return $installation
}

function Import-CppEnvironment($Installation) {
    $devcmd = Join-Path $Installation.installationPath 'Common7\Tools\VsDevCmd.bat'
    if (-not (Test-Path -LiteralPath $devcmd)) { throw 'VsDevCmd.bat nao encontrado.' }
    # cmd is used only for the vendor batch file and reading its environment.
    $commandLine = 'call "{0}" -no_logo -arch=x64 -host_arch=x64 >nul && set' -f $devcmd
    $lines = @(& $env:ComSpec /d /c $commandLine)
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao carregar o ambiente MSVC x64.' }
    foreach ($line in $lines) {
        if ($line -match '^([^=]+)=(.*)$') {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
        }
    }
}

function Get-WebViewRuntime {
    $appId = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    foreach ($hive in @('HKLM:\SOFTWARE\WOW6432Node', 'HKLM:\SOFTWARE', 'HKCU:\SOFTWARE')) {
        $key = "$hive\Microsoft\EdgeUpdate\Clients\$appId"
        if (-not (Test-Path -LiteralPath $key)) { continue }
        $version = Get-ItemPropertyValue -LiteralPath $key -Name pv -ErrorAction SilentlyContinue
        if ($version -and $version -ne '0.0.0.0') {
            $roots = @(
                (Join-Path ${env:ProgramFiles(x86)} 'Microsoft\EdgeWebView\Application'),
                (Join-Path $env:LOCALAPPDATA 'Microsoft\EdgeWebView\Application')
            )
            foreach ($root in $roots) {
                $exe = Join-Path (Join-Path $root $version) 'msedgewebview2.exe'
                if (Test-Path -LiteralPath $exe) {
                    return [pscustomobject]@{ Version = $version; Path = $exe; Registry = $key }
                }
            }
        }
    }
    return $null
}

function Get-TauriCli {
    $root = Get-EnvironmentRoot
    foreach ($candidate in @(
        (Join-Path $root '.tooling\environment-cli\node_modules\.bin\tauri.cmd'),
        (Join-Path $root 'node_modules\.bin\tauri.cmd')
    )) {
        if (Test-Path -LiteralPath $candidate) { return $candidate }
    }
    return $null
}

function Assert-OfficialInstaller([string]$Path, [string]$Publisher, [string]$Sha256 = '') {
    if ($Sha256 -and (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash -ne $Sha256) {
        throw "SHA-256 incorreto: $Path"
    }
    $signature = Get-AuthenticodeSignature -LiteralPath $Path
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch $Publisher) {
        throw "Assinatura oficial nao validada: $Path"
    }
}
