#requires -Version 5.1
<#!
.SYNOPSIS
Idempotent Windows 11 x64 tool setup. Never removes existing installations.
.PARAMETER InstallBuildTools
Explicitly authorizes the official C++ installer to request UAC if necessary.
Only that installer is elevated, not this script, npm, Rust or the doctor.
.PARAMETER SkipDoctor
Prepare tools without running the final build probes. Not an acceptance pass.
#>
[CmdletBinding()]
param([switch]$InstallBuildTools, [switch]$SkipDoctor)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'environment-common.ps1')
$root = Get-EnvironmentRoot
$evidence = Join-Path $root 'docs\evidence\environment'
$installers = Join-Path $root '.tooling\installers'
New-Item -ItemType Directory -Force -Path $evidence, $installers | Out-Null
$transcript = Join-Path $evidence 'bootstrap.log'
Start-Transcript -Path $transcript -Force | Out-Null
$bootstrapOk = $false
try {
    if ($env:OS -ne 'Windows_NT' -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') { throw 'Este bootstrap requer Windows x64 e PowerShell x64.' }
    Update-ProcessToolPath
    # TLS 1.2 is needed on Windows PowerShell 5.1; no remote scripts are evaluated.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    if (-not (Get-NativeTool 'git.exe')) {
        $winget = Get-NativeTool 'winget.exe'; if (-not $winget) { throw 'Instale Git pelo instalador oficial ou disponibilize winget.' }
        $null = Invoke-ToolChecked $winget @('install','--id','Git.Git','--exact','--source','winget','--scope','user','--silent','--accept-source-agreements','--accept-package-agreements') (Join-Path $evidence 'git-install.log')
        Add-UserToolPath (Join-Path $env:LOCALAPPDATA 'Programs\Git\cmd')
    } else { Write-Host 'Git existente preservado.' }

    if (-not (Get-NativeTool 'node.exe')) {
        # Official LTS portable archive: no machine installation or UAC required.
        $version = '24.22.1'
        $archiveName = "node-v$version-win-x64.zip"
        $base = "https://nodejs.org/dist/v$version"
        $archive = Join-Path $installers $archiveName
        $checksums = (Invoke-WebRequest -UseBasicParsing -Uri "$base/SHASUMS256.txt").Content
        $line = @($checksums -split "`n" | Where-Object { $_.TrimEnd().EndsWith("  $archiveName") })
        if ($line.Count -ne 1) { throw 'Checksum Node oficial nao localizado.' }
        $expectedHash = ($line[0] -split '\s+')[0]
        if (-not (Test-Path -LiteralPath $archive) -or (Get-FileHash $archive -Algorithm SHA256).Hash -ne $expectedHash) {
            Invoke-WebRequest -UseBasicParsing -Uri "$base/$archiveName" -OutFile $archive
        }
        if ((Get-FileHash $archive -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Checksum Node incorreto.' }
        $destination = Join-Path $env:LOCALAPPDATA 'Programs\SheetsMonitorTools'
        New-Item -ItemType Directory -Force -Path $destination | Out-Null
        $nodeDirectory = Join-Path $destination "node-v$version-win-x64"
        if (-not (Test-Path -LiteralPath (Join-Path $nodeDirectory 'node.exe'))) { Expand-Archive -LiteralPath $archive -DestinationPath $destination }
        Add-UserToolPath $nodeDirectory
    } else { Write-Host 'Node existente preservado; doctor verifica se e LTS compativel.' }

    $npm = Get-NativeTool 'npm.cmd'; if (-not $npm) { throw 'npm.cmd nao encontrado apos configurar Node.' }
    Add-UserToolPath (Join-Path $env:APPDATA 'npm')
    if (-not (Get-NativeTool 'pnpm.cmd')) {
        $null = Invoke-ToolChecked $npm @('install','--global','pnpm@10.34.9','--prefix',(Join-Path $env:APPDATA 'npm'),'--ignore-scripts','--no-fund','--no-audit') (Join-Path $evidence 'pnpm-install.log')
        Add-UserToolPath (Join-Path $env:APPDATA 'npm')
    } else { Write-Host 'pnpm existente preservado; doctor verifica major 10.' }

    Add-UserToolPath (Join-Path $env:USERPROFILE '.cargo\bin')
    if (-not (Get-NativeTool 'rustup.exe')) {
        $winget = Get-NativeTool 'winget.exe'; if (-not $winget) { throw 'winget necessario para instalar rustup com hash verificado.' }
        $null = Invoke-ToolChecked $winget @('install','--id','Rustlang.Rustup','--exact','--source','winget','--scope','user','--silent','--accept-source-agreements','--accept-package-agreements','--override','-y --profile minimal --no-modify-path --default-toolchain stable-x86_64-pc-windows-msvc') (Join-Path $evidence 'rustup-install.log')
        Add-UserToolPath (Join-Path $env:USERPROFILE '.cargo\bin')
    }
    $rustup = Get-NativeTool 'rustup.exe'; if (-not $rustup) { throw 'rustup nao encontrado.' }
    $toolchains = Invoke-ToolChecked $rustup @('toolchain','list') ''
    if ($toolchains -notmatch '(?m)^stable-x86_64-pc-windows-msvc') {
        $null = Invoke-ToolChecked $rustup @('toolchain','install','stable-x86_64-pc-windows-msvc','--profile','minimal') (Join-Path $evidence 'rust-toolchain-install.log')
    }
    $active = Invoke-ToolChecked $rustup @('show','active-toolchain') ''
    if ($active -notmatch '^stable-x86_64-pc-windows-msvc\s') {
        $null = Invoke-ToolChecked $rustup @('default','stable-x86_64-pc-windows-msvc') (Join-Path $evidence 'rust-default.log')
    }
    $null = Invoke-ToolChecked $rustup @('component','add','--toolchain','stable-x86_64-pc-windows-msvc','rustfmt','clippy') (Join-Path $evidence 'rust-components.log')

    if (-not (Get-TauriCli)) {
        $cliDirectory = Join-Path $root '.tooling\environment-cli'
        $null = Invoke-ToolChecked $npm @('install','--prefix',$cliDirectory,'@tauri-apps/cli@2.12.4','--ignore-scripts','--no-fund','--no-audit') (Join-Path $evidence 'tauri-cli-install.log')
    } else { Write-Host 'Tauri CLI existente preservada.' }

    if (-not (Get-WebViewRuntime)) {
        $webviewInstaller = Join-Path $installers 'MicrosoftEdgeWebview2Setup.exe'
        Invoke-WebRequest -UseBasicParsing -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $webviewInstaller
        Assert-OfficialInstaller $webviewInstaller 'CN=Microsoft Corporation'
        $process = Start-Process -FilePath $webviewInstaller -ArgumentList @('/silent','/install') -WindowStyle Hidden -Wait -PassThru
        if ($process.ExitCode -ne 0) { throw "WebView2 falhou ($($process.ExitCode)). Nenhum comando adicional foi elevado." }
    } else { Write-Host 'WebView2 existente preservado.' }

    $cpp = Get-CppInstallation
    $cppReady = $false
    if ($cpp) {
        Import-CppEnvironment $cpp
        $cppReady = [bool](Get-NativeTool 'cl.exe') -and [bool](Get-NativeTool 'link.exe') -and [bool](Get-NativeTool 'rc.exe') -and [bool]$env:WindowsSDKVersion
    }
    if (-not $cppReady) {
        if (-not $InstallBuildTools) { throw 'C++/SDK ausente. Execute .\scripts\bootstrap.ps1 -InstallBuildTools para autorizar somente o UAC do instalador Microsoft.' }
        if (Get-Process -Name vs_BuildTools -ErrorAction SilentlyContinue) { throw 'Um instalador Build Tools ja esta ativo. Aguarde sua conclusao antes de repetir o bootstrap.' }
        $installer = Join-Path $installers 'vs_BuildTools.exe'
        $hash = '5dc747350fb0589f908915c7b5fa52ec22d22b37ae2e5f4964d83519667498b2'
        $uri = "https://download.visualstudio.microsoft.com/download/pr/6bd63d7f-1818-4e03-a552-9feb11496c19/$hash/vs_BuildTools.exe"
        if (-not (Test-Path -LiteralPath $installer)) { Invoke-WebRequest -UseBasicParsing -Uri $uri -OutFile $installer }
        Assert-OfficialInstaller $installer 'CN=Microsoft Corporation' $hash
        # Explicit required tools avoid downloading optional IDE, LLVM and tests.
        $arguments = @('--passive','--wait','--norestart','--nocache','--add','Microsoft.VisualStudio.Workload.VCTools','--add','Microsoft.VisualStudio.Component.VC.Tools.x86.x64','--add','Microsoft.VisualStudio.Component.Windows11SDK.26100')
        if (-not $cpp) { $cpp = Get-CppInstallation -IncludeIncomplete }
        if ($cpp) { $arguments = @('modify','--installPath',('"{0}"' -f $cpp.installationPath)) + $arguments }
        Write-Host 'Somente o instalador oficial C++ solicitara UAC. Nenhuma instalacao sera removida.'
        $process = Start-Process -FilePath $installer -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -Wait -PassThru
        if ($process.ExitCode -eq 3010) { throw 'Instalacao concluida, mas Windows requer reinicio. Reinicie e rode doctor novamente.' }
        if ($process.ExitCode -ne 0) { throw "Instalacao C++ falhou ou foi cancelada: $($process.ExitCode)." }
    } else { Write-Host 'Visual Studio C++ e SDK existentes preservados.' }
    $bootstrapOk = $true
} catch {
    Write-Host "BOOTSTRAP FAIL: $($_.Exception.Message)"
} finally { Stop-Transcript | Out-Null }
if (-not $bootstrapOk) { exit 1 }
if ($SkipDoctor) { Write-Host 'Bootstrap finalizado. Rode doctor para validar o aceite.'; exit 0 }
& (Join-Path $PSScriptRoot 'doctor.ps1')
exit $LASTEXITCODE
