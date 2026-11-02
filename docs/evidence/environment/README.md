# Evidencias — Prompt 01

- `doctor.json`: resultado final estruturado; `ready_for_build: true`, todas as 16 verificacoes aprovadas.
- `doctor-first-full-run.log`: doctor direto sob PowerShell 7, encerrado em `DOCTOR PASS`.
- `bootstrap-powershell51.log`: bootstrap completo e doctor sob Windows PowerShell 5.1, encerrados com sucesso.
- `bootstrap-idempotent-run.log` e `bootstrap-repeat-path.log`: repeticao sem reinstalar ferramentas nem mudar PATH ja configurado.
- `cpp-build.log` / `cpp-run.log`, `resource-build.log`, `rust-build.log` / `rust-run.log`, `tauri-build.log` / `tauri-run.log`: compilacoes e execucoes minimas. Compiladores que retornam sucesso sem texto podem produzir log vazio ou nenhum arquivo de stdout; o resultado esta tambem no JSON e no transcript.
- `tauri-version.log` e `tauri-probe.Cargo.lock`: versao Tauri efetiva e resolucao de dependencias da probe, restaurada pelo doctor se o cache local nao existir.
- `powershell51-msvc.log`, `script-validation.log`, `diagnostic-guards.log`, `path-idempotence.log`: compatibilidade de shell, parser, rejeicao de subprocesso com exit 9, rejeicao de SHA-256 incorreto e PATH sem duplicacao.
- `vs-installer-verification.json`: origem, hash e assinatura Microsoft do instalador C++. `vs-install-first-errors.log` e `vs-resume-exit.txt`: cancelamentos reportados pelo instalador, preservados mesmo com o estado efetivo posterior instalado/completo e as provas aprovadas.
- `doctor-before-bootstrap.json`: falha esperada antes dos componentes C++/SDK/Rustfmt/Clippy.
- `doctor-first-tauri-failure.json`, `tauri-first-build.log`, `tauri-imports.log`: a primeira probe compilou, mas faltava manifesto Common Controls 6 para iniciar. Corrigido no doctor, sem alterar DLLs ou implementacao do aplicativo.

Houve tambem uma falha de aspas na primeira invocacao VsDevCmd; a chamada foi corrigida para `cmd /d /c call "...VsDevCmd.bat" ...` e verificada em PowerShell 5.1 e 7. O transcript final do bootstrap substitui sua captura inicial desse erro; este registro descreve a correcao observada, nao se apresenta como log bruto da falha.

Winget inspecionado: v1.29.380. Apenas o instalador oficial C++ foi elevado, com aprovacao explicita. Nao se executaram scripts remotos, nao houve desinstalacoes nem alteracao de ExecutionPolicy/antivirus.

O resultado comprova o ambiente de build. Nao comprova funcionalidades, instalador, interface nativa ou integracao Google do produto.
