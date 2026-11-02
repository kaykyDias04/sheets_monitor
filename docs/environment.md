# Ambiente de desenvolvimento

Verificado em 2026-11-01 23:49:20 -03:00 (America/Sao_Paulo).

Pronto para build: **True**. Exit code do doctor: 0.

Escopo: Prompt 01. As provas usam programas minimos isolados em `.tooling/environment-probe/`; nao implementam nem iniciam o aplicativo.

| Verificacao | Resultado | Versao / evidencia |
|---|---|---|
| Windows 11 x64 | OK | Microsoft Windows 11 Home; versao 10.0.26200; build 26200; x64 |
| PowerShell | OK | 5.1.26100.9711 Desktop |
| Git | OK | git version 2.53.0.windows.1 |
| Node LTS | OK | 24.15.0 LTS Krypton; C:\Program Files\nodejs\node.exe |
| npm | OK | 11.12.1 |
| pnpm 10 | OK | 10.34.9; C:\Users\kayky\AppData\Roaming\npm\pnpm.cmd |
| Rustup e Rust MSVC estavel | OK | rustc 1.99.1 (58cf8f9c3 2026-10-13); cargo 1.99.1 (5f94df478 2026-08-27); rustup 1.29.2 (d8c92c676 2026-10-21)<br>info: This is the version for the rustup toolchain manager, not the rustc compiler.<br>info: the currently active `rustc` version is `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; stable-x86_64-pc-windows-msvc (default) |
| rustfmt e clippy | OK | rustfmt e clippy instalados para MSVC |
| Visual Studio C++ | OK | Visual Studio 17.14.37827.9; MSVC 14.44.35207; cl 19.44.35229.0; C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools |
| Windows SDK | OK | 10.0.26100.0; C:\Program Files (x86)\Windows Kits\10\ |
| WebView2 Runtime | OK | 156.0.4314.58; C:\Program Files (x86)\Microsoft\EdgeWebView\Application\156.0.4314.58\msedgewebview2.exe |
| Tauri CLI 2 | OK | tauri-cli 2.12.4 |
| Build C++ e link Windows | OK | cpp-sdk-ok |
| Windows resource compiler | OK | rc.exe gerou resource.res |
| Build e execucao Rust MSVC | OK | rust-msvc-ok |
| Build e execucao minima Tauri 2 | OK | tauri v2.12.5<br>[build-dependencies]; tauri2-build-ok |

## Reproduzir

```powershell
.\scripts\bootstrap.ps1
.\scripts\doctor.ps1
```

Se C++ estiver ausente, execute `bootstrap.ps1 -InstallBuildTools` e aprove somente o UAC do instalador Microsoft. O script principal e as verificacoes permanecem sem elevacao. Nenhuma instalacao preexistente e removida.

PATH persistente: bootstrap adiciona apenas diretorios de ferramentas do usuario ausentes, sem substituir o PATH da maquina. VsDevCmd e importado somente no processo das verificacoes; cl/link/rc nao sao adicionados globalmente.

Evidencias: [doctor.json](evidence/environment/doctor.json), logs de compilacao/execucao no mesmo diretorio. Ver [politica e fontes](environment-policy.md). O doctor nao valida funcionalidades, OAuth, baseline ou empacotamento do produto.
