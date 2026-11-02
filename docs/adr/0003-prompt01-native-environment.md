# ADR 0003 — ambiente nativo e prova isolada

Status: aceito no Prompt 01.

O contrato mestre separa o desenvolvimento em prompts. Esta etapa prepara somente o ambiente de desenvolvimento Windows, sem continuar a implementacao iniciada sob a autorizacao anterior.

Adotamos Rust stable MSVC, Visual Studio Build Tools 2022 estavel e Windows SDK 26100. A selecao de cl/link/rc vem de VsDevCmd, restrita ao processo de verificacao. Toolchains GNU e outras instalacoes preexistentes permanecem intactas; nao contam como prova do caminho MSVC.

Preservamos Node LTS existente e instalamos pnpm pelo npm no perfil do usuario. Nao migramos manifestos nem lockfiles do aplicativo no Prompt 01. O PATH do usuario recebe apenas diretorios ausentes; nao se altera PATH de maquina nem politica de execucao PowerShell.

Um inventario de versoes nao demonstra que os headers, import libraries e linker funcionam. Por isso, doctor compila e executa probes isoladas C++/SDK, recursos, Rust e Tauri/Wry em `.tooling/environment-probe/`. A probe Tauri nao abre janela nem implementa comportamento do aplicativo. Logs, versoes e lockfile da probe ficam em `docs/evidence/environment/`.

Somente o instalador oficial C++ recebeu elevacao, apos consentimento explicito do usuario. Seu SHA-256 e assinatura foram verificados. Downloads nao sao executados como scripts remotos. Falhas e cancelamentos de instalacao sao registrados; o aceite depende do doctor completo, nao do codigo de saida isolado do instalador.

Ao concluir o aceite, registrar progresso e aguardar Prompt 02.
