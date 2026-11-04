# Sprint 06 — retomada 06.1

**PENDENTE DE HOMOLOGAÇÃO**, em 03/11/2026. SM-026–SM-028 não estão concluídos. Checkpoint: [SPRINT_06_PROGRESS.md](SPRINT_06_PROGRESS.md). Resultados estruturados: [final-validation.json](evidence/sprint06/final-validation.json).

Base limpa `b2c68ce`, branch `codex/sprint-06-homologation`. Nenhum trabalho anterior da Sprint 06 ou instalador recuperado. Prompt preservado em `sprint06.1.original.md`, SHA256 `82531235AE49B3EA2C2C595FFC21B03C2EA93849DBFD667CAB4E9E0D8B2EA072`.

| Entrega | Resultado e limite |
|---|---|
| SM-026 | Matriz incremental, 258 unitários Rust, duas integrações e 50 React aprovados. Um teste de cofre ignorado por padrão, implementação inalterada. Leituras/refresh/cofre reais históricos preservados; nenhuma nova homologação Google nesta máquina |
| SM-027 | 30 arquivos sintéticos, 90 verificações, máximo três paralelas, 30 falhas de cota e 30 recuperações; intervalos 60/300. Agendador/SQLite reais com relógio, transporte e conta sintéticos. Não é soak nem vazão Google |
| Métricas | Processo de teste: 4772 ms, pico observado working set 14798848 bytes, private máximo amostrado 4190208 bytes; amostragem 100 ms. Não representa memória do desktop completo |
| SM-028 | Build produção x64 GUI e NSIS aprovados. Instalador técnico `0.1.0`, 4946164 bytes, sem assinatura. Instalação/desinstalação e persistência instalada não executadas: solicitação de execução recusada |
| Avisos | 389 pacotes e avisos oficiais da biblioteca padrão Rust; zero textos ausentes pelo coletor. Inventário conservador, incluindo build/dev; aviso MIT de react-remove-scroll-bar obtido no upstream atual, não no commit publicado. Limite registrado, sem alegar auditoria jurídica |

Artefato: `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Sheets Monitor_0.1.0_x64-setup.exe`. SHA256: `AFE8F45A09276A7948A0C41F4102A5091BCC71324967A6E0A2E09C7C157D8527`. Hashes/tamanhos em [artifact-manifest.json](evidence/sprint06/artifact-manifest.json). Binários locais ignorados pelo Git; nenhuma release publicada. A arquitetura e as importações constam dos logs `production-pe-headers.log` e `production-dependencies.log`, sem dependência direta de DLLs Node/Rust/MSVC de desenvolvimento. Isso não comprova execução em imagem Windows limpa.

Comandos aprovados: `npm ci --prefer-offline --no-audit --no-fund`, `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`, `npm run build`, `scripts/test-load.ps1`, `scripts/check-rust.ps1 -Task test`, `-Task clippy`, `-Task format-check`, `node scripts/distribution-notices.mjs`, `node scripts/verify-repository.mjs`, `npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis`. Logs em `docs/evidence/sprint06/`.

Correções: normalização LF das fontes em checkout Windows (falha inicial Prettier em 63 arquivos, sem mudança semântica frontend), sincronização dos backlogs e separação do destino de novas evidências. Nenhum defeito de produto adicional foi comprovado. Dependências restauradas por lockfile; Rust/C++/SDK instalados com autorização humana, sem sobrescrever provas de ambiente anteriores.

Pendências: configuração OAuth da aplicação ausente; campanha Google real com 30 recursos autorizados e referências humanas compatíveis; instalação/abertura independente/persistência/desinstalação; identidade instalada de notificações, interação física e autostart no login. O pacote técnico não tem login provisionado. Solicitação ao usuário pede somente caminho local seguro do arquivo OAuth Desktop do projeto autorizado, nunca tokens/senhas/conteúdo no chat. Mudança de configuração de empacotamento exigirá novo build.

Nenhuma escrita Google, aprovação de baseline, ativação de autostart ou envio de novo toast real. Encerrar a execução técnica disponível e aguardar intervenção para os bloqueios; não iniciar V1.1.

## Atualização autorizada — instalação e pré-release

O usuário autorizou expressamente instalar o pacote e publicar a release no GitHub nesta conversa. Essa autorização substitui a recusa de instalação anterior e a restrição anterior de publicação para este pacote técnico.

Instalado por usuário em `%LOCALAPPDATA%\Programs\Sheets Monitor`. Processo permaneceu ativo após abertura com Node/Rust/MSVC fora do PATH; avisos de terceiros, biblioteca padrão Rust e manual presentes. A comparação integral do payload confirmou diferença exclusiva do marcador Tauri `NSS` (NSIS) versus `UNK` (executável solto); hashes e resultado em `docs/evidence/sprint06/installed-payload-validation.json` e `install-authorized-result.json`. A comparação inicial de hashes falhou por esse marcador; não houve alteração de código ou rebuild.

Pré-release publicada: https://github.com/kaykyDias04/sheets_monitor/releases/tag/v0.1.0-sprint06-preview . Instalador e SHA256SUMS.txt enviados, com digests retornados pelo GitHub conferidos. Registro em `docs/evidence/sprint06/github-release.json`. Tag aponta para `f19f1ad`, fonte do pacote. Não é aprovação da V1.0: OAuth ausente, carga Google real, persistência após reinício, desinstalação e homologação Windows completa continuam pendentes. Nenhuma baseline aprovada, autostart ativado ou arquivo Google alterado.

Próxima ação: disponibilizar a configuração OAuth da aplicação por caminho local seguro e concluir a homologação pendente. Preservar o pacote instalado; não reinstalar nem reconstruir sem necessidade.


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.
