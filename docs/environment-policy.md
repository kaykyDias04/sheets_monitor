# Politica de ambiente — Prompt 01

Escopo autorizado: inspecao e preparacao do Windows 11 para build Tauri 2. Nenhum modulo ou funcionalidade do aplicativo e implementado nesta etapa. Arquivos anteriores ao Prompt 00 permanecem preservados, sem que isso declare sua conclusao.

O responsavel pela verificacao e o agente principal. O aceite requer `scripts/doctor.ps1` retornar 0, inventario em `docs/environment.md` e evidencias de compilacao/execucao de probes isoladas C++, recursos Windows, Rust MSVC e Tauri 2. Testar apenas `--version` seria evidencia insuficiente para demonstrar headers, bibliotecas e linker funcionando.

## Versoes e instalacao

- Preservar Git e Node LTS existentes se compativeis. Politica Node: LTS 22.12+, 24 ou 26 em Windows x64, conferindo `process.release.lts`. Ausencia de Node: ZIP LTS 24.22.1 oficial, SHA-256 conferido, instalado no perfil do usuario.
- pnpm 10.x; instalacao ausente fixa 10.34.9 via npm oficial, sem lifecycle scripts e sem migrar o lockfile do projeto nesta etapa.
- Rustup com toolchain `stable-x86_64-pc-windows-msvc`, rustfmt e clippy. Toolchains GNU existentes sao preservadas e nao contam como aceite do Windows MSVC.
- Visual Studio Build Tools 2022 estavel com `Microsoft.VisualStudio.Workload.VCTools`, `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` e `Microsoft.VisualStudio.Component.Windows11SDK.26100` explicitos. Bootstrapper oficial obtido do manifesto winget 17.14.44, SHA-256 fixado e Authenticode Microsoft valido antes de executar. Os componentes servidos pelo canal estavel sao registrados pelo doctor apos instalacao. A primeira tentativa com componentes recomendados foi cancelada durante downloads; a retomada solicita somente os componentes explicitos necessarios, preservando os ja instalados.
- WebView2 Evergreen existente e preservado. Ausencia: bootstrapper oficial Microsoft, assinatura verificada, instalacao por usuario sem elevar o script principal.
- Tauri CLI 2 existente; ausencia instala 2.12.4 isoladamente em `.tooling/environment-cli/`. O probe Cargo usa Tauri 2 e conserva Cargo.lock apos a primeira resolucao bem sucedida.

Nenhum script remoto e baixado e executado por `Invoke-Expression`, pipe para shell ou mecanismo equivalente. Downloads sao arquivos oficiais de instalacao/arquivo ZIP, validados antes da execucao. npm/winget verificam a integridade dos seus pacotes. As verificacoes nao desativam antivirus, nao alteram ExecutionPolicy e nao removem instalacoes existentes.

## PATH e privilegios

Bootstrap acrescenta `.cargo/bin` e o prefixo npm do usuario ao PATH do usuario apenas se ausentes. O PATH de maquina nao e alterado. O ambiente MSVC e carregado via `VsDevCmd.bat` somente no processo que executa as provas; nao adicionar `cl.exe` globalmente evita selecionar SDK/linker incorreto.

`-InstallBuildTools` e a autorizacao explicita para solicitar UAC somente no executavel oficial C++. O script principal, Node/npm/pnpm, Rust e doctor continuam no contexto do usuario. Um reinicio solicitado pelo instalador impede declarar aceite ate nova execucao do doctor. Instalacoes que nao podem ser preservadas/configuradas com seguranca geram falha com orientacao em vez de desinstalar ferramentas.

O doctor atualiza somente arquivos de diagnostico e probes locais; nao corrige o sistema nem instala dependencias. O bootstrap pode ser repetido: verifica presenca antes de instalar, nao duplica entradas de PATH e executa doctor ao terminar.

## Fontes oficiais consultadas

- [Pre-requisitos Tauri 2 para Windows](https://v2.tauri.app/start/prerequisites/).
- [Ciclo de suporte Node.js](https://nodejs.org/en/about/previous-releases) e [cronograma oficial](https://github.com/nodejs/Release).
- [Instalacao pnpm no Windows](https://pnpm.io/installation).
- [Parametros do instalador Visual Studio](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=vs-2022).
- [Distribuicao e deteccao WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).

## Limites

O probe nao abre uma janela, nao exerce OAuth e nao gera instalador do produto. Um resultado positivo demonstra compilacao nativa com Tauri/Wry, headers C++ e SDK, linker MSVC, resource compiler e runtime WebView2 instalado. Funcionalidades e empacotamento pertencem aos prompts posteriores.

A prova inclui um manifesto Windows Common Controls 6, equivalente a dependencia de SO inserida normalmente por tauri-build. A primeira prova sem manifesto compilou, mas o loader retornou `0xC0000139` ao resolver `TaskDialogIndirect` em comctl32 v5; o manifesto corrigiu a prova e a execucao passou. Nao foi necessario substituir DLLs do sistema nem reinstalar runtimes.
