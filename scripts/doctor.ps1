#requires -Version 5.1
<#!
.SYNOPSIS
Checks Windows 11 x64 and compiles isolated C++, resource, Rust and Tauri 2 probes.
.DESCRIPTION
Does not install software or modify persistent PATH. Evidence is written in docs/.
Exit 0 means all required checks/probes passed; exit 1 means not build-ready.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'environment-common.ps1')
$root = Get-EnvironmentRoot
$evidence = Join-Path $root 'docs\evidence\environment'
$probe = Join-Path $root '.tooling\environment-probe'
New-Item -ItemType Directory -Force -Path $evidence, $probe | Out-Null
Update-ProcessToolPath
$results = [System.Collections.Generic.List[object]]::new()

function Check([string]$Name, [scriptblock]$Action) {
    try {
        $detail = (& $Action | Out-String).Trim()
        $results.Add([pscustomobject]@{ name = $Name; ok = $true; detail = $detail })
        Write-Host "[OK] $Name : $detail"
    } catch {
        $detail = $_.Exception.Message
        $results.Add([pscustomobject]@{ name = $Name; ok = $false; detail = $detail })
        Write-Host "[FALHA] $Name : $detail"
    }
}

Check 'Windows 11 x64' {
    $os = Get-CimInstance Win32_OperatingSystem
    if ([int]$os.BuildNumber -lt 22000 -or $os.Caption -notmatch 'Windows 11') { throw 'Windows 11 requerido.' }
    if ($env:PROCESSOR_ARCHITECTURE -ne 'AMD64' -or -not [Environment]::Is64BitProcess) { throw 'Execute PowerShell x64 em Windows x64.' }
    "$($os.Caption); versao $($os.Version); build $($os.BuildNumber); x64"
}
Check 'PowerShell' { "$($PSVersionTable.PSVersion) $($PSVersionTable.PSEdition)" }
Check 'Git' {
    $git = Get-NativeTool 'git.exe'; if (-not $git) { throw 'Git nao encontrado no PATH.' }
    Invoke-ToolChecked $git @('--version') ''
}
Check 'Node LTS' {
    $node = Get-NativeTool 'node.exe'; if (-not $node) { throw 'Node nao encontrado no PATH.' }
    $raw = Invoke-ToolChecked $node @('-p', 'JSON.stringify({version:process.versions.node,lts:process.release.lts,arch:process.arch})') ''
    $info = $raw | ConvertFrom-Json
    # Supported LTS lines in the project environment policy (November 2026).
    if (-not $info.lts -or ([version]$info.version).Major -notin @(22,24,26) -or [version]$info.version -lt [version]'22.12.0' -or $info.arch -ne 'x64') {
        throw "Node LTS x64 suportado requerido (22.12+, 24 ou 26): $raw"
    }
    "$($info.version) LTS $($info.lts); $node"
}
Check 'npm' {
    $npm = Get-NativeTool 'npm.cmd'; if (-not $npm) { throw 'npm.cmd ausente.' }
    Invoke-ToolChecked $npm @('--version') ''
}
Check 'pnpm 10' {
    $pnpm = Get-NativeTool 'pnpm.cmd'; if (-not $pnpm) { throw 'pnpm.cmd nao encontrado.' }
    $version = Invoke-ToolChecked $pnpm @('--version') ''
    if ($version -notmatch '^10\.\d+\.\d+$') { throw "pnpm estavel 10.x requerido: $version" }
    "$version; $pnpm"
}
Check 'Rustup e Rust MSVC estavel' {
    $rustup = Get-NativeTool 'rustup.exe'; if (-not $rustup) { throw 'rustup ausente.' }
    $active = Invoke-ToolChecked $rustup @('show','active-toolchain') ''
    if ($active -notmatch '^stable-x86_64-pc-windows-msvc\s') { throw "Toolchain ativa precisa ser stable MSVC: $active" }
    $rust = Invoke-ToolChecked (Get-NativeTool 'rustc.exe') @('--version') ''
    $cargo = Invoke-ToolChecked (Get-NativeTool 'cargo.exe') @('--version') ''
    $up = Invoke-ToolChecked $rustup @('--version') ''
    "$rust; $cargo; $up; $active"
}
Check 'rustfmt e clippy' {
    $installed = Invoke-ToolChecked (Get-NativeTool 'rustup.exe') @('component','list','--installed') ''
    if ($installed -notmatch 'rustfmt-x86_64-pc-windows-msvc' -or $installed -notmatch 'clippy-x86_64-pc-windows-msvc') { throw 'Execute bootstrap para instalar rustfmt e clippy.' }
    'rustfmt e clippy instalados para MSVC'
}
Check 'Visual Studio C++' {
    $installation = Get-CppInstallation
    if (-not $installation) { throw 'Visual Studio C++ completo nao encontrado; execute bootstrap -InstallBuildTools.' }
    if ($installation.isRebootRequired) { throw 'Visual Studio requer reiniciar o Windows antes do aceite.' }
    Import-CppEnvironment $installation
    foreach ($tool in @('cl.exe','link.exe','rc.exe')) { if (-not (Get-NativeTool $tool)) { throw "$tool ausente apos VsDevCmd." } }
    $compilerVersion = (Get-Item -LiteralPath (Get-NativeTool 'cl.exe')).VersionInfo.FileVersion
    "Visual Studio $($installation.installationVersion); MSVC $env:VCToolsVersion; cl $compilerVersion; $($installation.installationPath)"
}
Check 'Windows SDK' {
    if (-not $env:WindowsSdkDir -or -not $env:WindowsSDKVersion) { throw 'Windows SDK nao carregado.' }
    $sdk = Join-Path $env:WindowsSdkDir ('Lib\' + $env:WindowsSDKVersion.TrimEnd('\'))
    foreach ($relative in @('um\x64\kernel32.lib', 'ucrt\x64\ucrt.lib')) {
        if (-not (Test-Path -LiteralPath (Join-Path $sdk $relative))) { throw "SDK incompleto: $relative" }
    }
    "$($env:WindowsSDKVersion.TrimEnd('\')); $env:WindowsSdkDir"
}
Check 'WebView2 Runtime' {
    $runtime = Get-WebViewRuntime
    if (-not $runtime) { throw 'WebView2 Runtime ausente ou registro sem executavel correspondente.' }
    "$($runtime.Version); $($runtime.Path)"
}
Check 'Tauri CLI 2' {
    $cli = Get-TauriCli; if (-not $cli) { throw 'Tauri CLI ausente; execute bootstrap.' }
    $version = Invoke-ToolChecked $cli @('--version') ''
    if ($version -notmatch '^tauri-cli 2\.') { throw "Tauri 2 requerido: $version" }
    $version
}

$prerequisitesOk = @($results | Where-Object { -not $_.ok }).Count -eq 0
Check 'Build C++ e link Windows' {
    if (-not $prerequisitesOk) { throw 'Prova nao executada: pre-requisitos ausentes.' }
    $source = Join-Path $probe 'cpp.cpp'; $exe = Join-Path $probe 'cpp.exe'; $obj = Join-Path $probe 'cpp.obj'
    @('#include <windows.h>','#include <cstdio>','int main(){ std::puts("cpp-sdk-ok"); return sizeof(void*) == 8 ? 0 : 1; }') | Set-Content -LiteralPath $source -Encoding ascii
    $null = Invoke-ToolChecked (Get-NativeTool 'cl.exe') @('/nologo','/EHsc',"/Fe$exe","/Fo$obj",$source) (Join-Path $evidence 'cpp-build.log')
    Invoke-ToolChecked $exe @() (Join-Path $evidence 'cpp-run.log')
}
Check 'Windows resource compiler' {
    if (-not $prerequisitesOk) { throw 'Prova nao executada: pre-requisitos ausentes.' }
    $source = Join-Path $probe 'resource.rc'; $binary = Join-Path $probe 'resource.res'
    '1 RCDATA { 1, 2, 3, 4 }' | Set-Content -LiteralPath $source -Encoding ascii
    $null = Invoke-ToolChecked (Get-NativeTool 'rc.exe') @('/nologo',"/fo$binary",$source) (Join-Path $evidence 'resource-build.log')
    if (-not (Test-Path -LiteralPath $binary)) { throw 'Arquivo .res nao gerado.' }
    'rc.exe gerou resource.res'
}
Check 'Build e execucao Rust MSVC' {
    if (-not $prerequisitesOk) { throw 'Prova nao executada: pre-requisitos ausentes.' }
    $source = Join-Path $probe 'rust.rs'; $exe = Join-Path $probe 'rust.exe'
    'fn main() { println!("rust-msvc-ok"); }' | Set-Content -LiteralPath $source -Encoding ascii
    $null = Invoke-ToolChecked (Get-NativeTool 'rustc.exe') @('--target','x86_64-pc-windows-msvc',$source,'-o',$exe) (Join-Path $evidence 'rust-build.log')
    Invoke-ToolChecked $exe @() (Join-Path $evidence 'rust-run.log')
}
Check 'Build e execucao minima Tauri 2' {
    if (-not $prerequisitesOk) { throw 'Prova nao executada: pre-requisitos ausentes.' }
    $tauriProbe = Join-Path $probe 'tauri'
    New-Item -ItemType Directory -Force -Path (Join-Path $tauriProbe 'src') | Out-Null
    @('[package]','name = "sheets-monitor-environment-probe"','version = "0.0.0"','edition = "2021"','','[workspace]','','[dependencies]','tauri = "2"') | Set-Content -LiteralPath (Join-Path $tauriProbe 'Cargo.toml') -Encoding ascii
    'fn main() { let _builder = tauri::Builder::<tauri::Wry>::default(); println!("tauri2-build-ok"); }' | Set-Content -LiteralPath (Join-Path $tauriProbe 'src\main.rs') -Encoding ascii
    # Wry imports TaskDialogIndirect: Windows requires Common Controls v6.
    # A real Tauri app embeds this via tauri-build. The isolated probe embeds
    # the same OS dependency without creating an application scaffold.
    $manifest = @'
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency><dependentAssembly>
    <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
  </dependentAssembly></dependency>
</assembly>
'@
    $manifest | Set-Content -LiteralPath (Join-Path $tauriProbe 'windows.manifest') -Encoding ascii
    $probeBuild = @'
fn main() {
    let path = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("windows.manifest");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", path.display());
    println!("cargo:rerun-if-changed=windows.manifest");
}
'@
    $probeBuild | Set-Content -LiteralPath (Join-Path $tauriProbe 'build.rs') -Encoding ascii
    $lock = Join-Path $tauriProbe 'Cargo.lock'
    $recordedLock = Join-Path $evidence 'tauri-probe.Cargo.lock'
    if (-not (Test-Path -LiteralPath $lock) -and (Test-Path -LiteralPath $recordedLock)) { Copy-Item -LiteralPath $recordedLock -Destination $lock }
    $cargoArguments = @('build','--manifest-path',(Join-Path $tauriProbe 'Cargo.toml'),'--target','x86_64-pc-windows-msvc','--target-dir',(Join-Path $tauriProbe 'target'))
    if (Test-Path -LiteralPath (Join-Path $tauriProbe 'Cargo.lock')) { $cargoArguments += '--locked' }
    Write-Host 'Compilando probe Tauri isolada (primeira execucao pode levar varios minutos)...'
    $null = Invoke-ToolChecked (Get-NativeTool 'cargo.exe') $cargoArguments (Join-Path $evidence 'tauri-build.log')
    $exe = Join-Path $tauriProbe 'target\x86_64-pc-windows-msvc\debug\sheets-monitor-environment-probe.exe'
    $execution = Invoke-ToolChecked $exe @() (Join-Path $evidence 'tauri-run.log')
    $version = Invoke-ToolChecked (Get-NativeTool 'cargo.exe') @('tree','--manifest-path',(Join-Path $tauriProbe 'Cargo.toml'),'--offline','--locked','--depth','0','-p','tauri') (Join-Path $evidence 'tauri-version.log')
    Copy-Item -LiteralPath $lock -Destination $recordedLock -Force
    "$version; $execution"
}

$success = @($results | Where-Object { -not $_.ok }).Count -eq 0
$stamp = [TimeZoneInfo]::ConvertTimeBySystemTimeZoneId([DateTimeOffset]::UtcNow, 'E. South America Standard Time').ToString('yyyy-MM-dd HH:mm:ss zzz')
$report = [ordered]@{ checked_at = $stamp; ready_for_build = $success; scope = 'Prompt 01 - ambiente; probes isoladas, sem build/execucao do aplicativo'; checks = @($results.ToArray()) }
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $evidence 'doctor.json') -Encoding utf8
$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add('# Ambiente de desenvolvimento'); $lines.Add(''); $lines.Add("Verificado em $stamp (America/Sao_Paulo)."); $lines.Add('')
$lines.Add("Pronto para build: **$success**. Exit code do doctor: $(if ($success) { 0 } else { 1 })."); $lines.Add('')
$lines.Add('Escopo: Prompt 01. As provas usam programas minimos isolados em `.tooling/environment-probe/`; nao implementam nem iniciam o aplicativo.'); $lines.Add('')
$lines.Add('| Verificacao | Resultado | Versao / evidencia |'); $lines.Add('|---|---|---|')
foreach ($check in $results) { $detail = $check.detail.Replace('|','/').Replace("`r",'').Replace("`n",'<br>'); $lines.Add("| $($check.name) | $(if ($check.ok) {'OK'} else {'FALHA'}) | $detail |") }
$lines.Add(''); $lines.Add('## Reproduzir'); $lines.Add(''); $lines.Add('```powershell'); $lines.Add('.\scripts\bootstrap.ps1'); $lines.Add('.\scripts\doctor.ps1'); $lines.Add('```')
$lines.Add(''); $lines.Add('Se C++ estiver ausente, execute `bootstrap.ps1 -InstallBuildTools` e aprove somente o UAC do instalador Microsoft. O script principal e as verificacoes permanecem sem elevacao. Nenhuma instalacao preexistente e removida.');
$lines.Add(''); $lines.Add('PATH persistente: bootstrap adiciona apenas diretorios de ferramentas do usuario ausentes, sem substituir o PATH da maquina. VsDevCmd e importado somente no processo das verificacoes; cl/link/rc nao sao adicionados globalmente.');
$lines.Add(''); $lines.Add('Evidencias: [doctor.json](evidence/environment/doctor.json), logs de compilacao/execucao no mesmo diretorio. Ver [politica e fontes](environment-policy.md). O doctor nao valida funcionalidades, OAuth, baseline ou empacotamento do produto.');
$lines | Set-Content -LiteralPath (Join-Path $root 'docs\environment.md') -Encoding utf8
if ($success) { Write-Host 'DOCTOR PASS: ambiente apto ao build Windows MSVC/Tauri 2.'; exit 0 }
Write-Host 'DOCTOR FAIL: veja docs/environment.md e docs/evidence/environment/doctor.json.'; exit 1
