# Sprint 06 — checkpoint de retomada

Atualizado em 03/11/2026. **PENDENTE DE HOMOLOGAÇÃO**. Autorização: Prompt 06.1 recebido como anexo nesta conversa. Branch `codex/sprint-06-homologation`, base `b2c68ce`. Escopo exclusivo SM-026–SM-028; sem publicação de release. Relatório consolidado: `docs/sprint-06-report.md`; estado estruturado: `docs/evidence/sprint06/final-validation.json`.

## Estado recuperado

- Git inicialmente limpo em `main`; nenhuma branch, evidência, checkpoint ou instalador da Sprint 06 localizado. Não existe `docs/sprint06.original.md` neste checkout. O Prompt 06.1 e a especificação v1.0 são as referências disponíveis; não presumir duração de campanha ausente.
- Reutilizar evidências históricas da Sprint 05: `docs/evidence/sprint05/local-validation.json` (257 unitários Rust, duas integrações, 50 React; um teste de cofre ignorado). São provas anteriores, não execuções desta retomada.
- Build anterior somente desenvolvimento, conforme `docs/evidence/sprint05/build-validation.json`. Sem `node_modules`, `src-tauri/target` ou bundle local recuperável. Configuração de bundle desativada.
- Recursos reais: último estado comprovado com quatro cadastros pendentes e zero escopos elegíveis. Nenhuma baseline será aprovada pelo agente.

## Pendências

- SM-026: matriz e regressões locais aprovadas; nova homologação Google/Windows instalada bloqueada por configuração e permissão ausentes.
- SM-027: campanha de 30 arquivos, métricas e recuperação; carga Google real depende de recursos autorizados e referências aprovadas pelo usuário.
- SM-028: produção/NSIS técnico aprovados; homologação instalação/persistência/desinstalação não executada após recusa da solicitação de execução. Revisão final de avisos conserva limite de procedência documentado.
- Ambiente: checkout atual no perfil `kawan`; a prova histórica de ambiente usa `kayky` e não valida esta máquina. Dependências npm restauradas pelo lockfile (304 pacotes). Usuário autorizou expressamente instalar Rust/C++ e eventual UAC.
- Rust 1.99.1 instalado pela distribuição oficial com checksum SHA256 conferido. Tentativa anterior via winget falhou com `No applicable installer found`. Rustfmt/Clippy instalados. C++/SDK oficial com assinatura/hash verificados concluiu com código 0 (`msvc-install.log`); instalação completa detectada pelo helper.

## Incrementos desta retomada

- `npm run typecheck`, `npm run lint` e `npm test`: aprovados; 50 testes em dez arquivos (`docs/evidence/sprint06/`).
- `npm run format:check`: falhou inicialmente em 63 arquivos por finais de linha CRLF do checkout. `npm run format` normalizou sem diff semântico; nova verificação aprovada em `format-final.log`. `.gitattributes` fixa LF nas fontes/frontend, preservando arquivo de protótipo e evidências históricas.
- Campanha determinística `thirty_workbooks_queue_quota_recovery_and_persistence` aprovada: 30 arquivos sintéticos, 90 verificações, fila global de três, 30 respostas de cota, embargo persistido e 30 recuperações, intervalos 60/300. `load-test.log`, `load-run.log` e `load-metrics.log`. Não é campanha Google real ou soak.
- Matriz incremental: `docs/evidence/sprint06/scenario-matrix.md`.
- Bundle NSIS habilitado na configuração; não existe instalador gerado nem homologado ainda.
- Cargo restaurado sem alterar lockfile; build frontend aprovado em `frontend-build.log`. Primeira compilação nativa concluída. `scripts/test-load.ps1` mediu o processo sintético isolado: 4772 ms, pico observado working set 14798848 bytes e máximo amostrado private 4190208 bytes, amostragem 100 ms (`load-validation.json`). Não são métricas do desktop completo. Regressão final: 258 unitários Rust, duas integrações, um cofre ignorado por padrão (inalterado); Clippy e rustfmt aprovados. Logs `rust-test.log`, `rust-clippy.log`, `rust-format-check.log`.
- Verificador documental inicialmente falhou por backlog Markdown não sincronizado; corrigido. Nova execução aprovada em `repository-check-final.log`, com hashes de protótipo/especificação e varredura de padrões conhecidos. Não é auditoria completa de segredos.
- Avisos coletados: 389 pacotes, zero lacunas de texto pelo coletor (`distribution-licenses.json`, `notices.log`). Inventário conservador inclui build/dev Rust; não é auditoria jurídica ou composição exata. Avisos suplementares em `supplemental-licenses.json`: `react-remove-scroll-bar` usa aviso MIT atual do mantenedor, não obtido no gitHead/tag publicado. Não ocultar esse limite de procedência.
- Avisos oficiais da biblioteca padrão Rust incluídos em `distribution/rust-notices/`; sem binários de toolchain. Coletor final em `notices-final.log`.
- Configuração OAuth de empacotamento ausente (`packaging-readiness.json`), sem diretório privado recuperado. Solicitado ao usuário somente caminho local seguro, nunca conteúdo/segredos no chat. Gerado build técnico NSIS (`production-build.log`), com login/homologação Google pendentes; não chamar o pacote de release aprovado. Build com configuração futura poderá exigir nova compilação.

## Último build válido e bloqueio de instalação

- Build NSIS técnico concluído uma vez, sem alterações posteriores dos seus recursos/código. Log `production-build.log`. Versão existente `0.1.0`, x64 GUI, não assinado.
- Instalador: `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Sheets Monitor_0.1.0_x64-setup.exe`, 4946164 bytes, SHA256 `AFE8F45A09276A7948A0C41F4102A5091BCC71324967A6E0A2E09C7C157D8527`. Executável/hash separado em `artifact-manifest.json`.
- Não houve instalação: a solicitação de execução foi rejeitada pelo usuário, comando inteiro não executado. Não tentar novamente sem nova orientação. `install-result.json` registra o limite; não afirmar startup independente, persistência instalada ou desinstalação. Não foi necessário sobrescrever instalação/dados reais.
- `production-pe-headers.log` e `production-dependencies.log`: binário gráfico x64, imports de sistema; não equivale a teste numa imagem Windows limpa.
- Não houve novas consultas Google, toast real, autostart ou aprovação de baseline. Nenhuma release publicada. Preservar o artefato técnico para revisão; não recompilar sem mudança que o invalide.

## Próxima ação

Aguardar intervenção: revisão/instalação manual autorizada do pacote técnico e caminho local seguro da configuração OAuth Desktop do projeto autorizado; depois prover recursos de leitura e referências aprovadas humanamente para a campanha real. Concluir homologação instalada/notificações/autostart quando autorizada. Não repetir os gates locais aprovados sem alteração relevante; não iniciar V1.1.

## Atualização autorizada — instalação e pré-release

O usuário autorizou expressamente instalar o pacote e publicar a release no GitHub nesta conversa. Essa autorização substitui a recusa de instalação anterior e a restrição anterior de publicação para este pacote técnico.

Instalado por usuário em `%LOCALAPPDATA%\Programs\Sheets Monitor`. Processo permaneceu ativo após abertura com Node/Rust/MSVC fora do PATH; avisos de terceiros, biblioteca padrão Rust e manual presentes. A comparação integral do payload confirmou diferença exclusiva do marcador Tauri `NSS` (NSIS) versus `UNK` (executável solto); hashes e resultado em `docs/evidence/sprint06/installed-payload-validation.json` e `install-authorized-result.json`. A comparação inicial de hashes falhou por esse marcador; não houve alteração de código ou rebuild.

Pré-release publicada: https://github.com/kaykyDias04/sheets_monitor/releases/tag/v0.1.0-sprint06-preview . Instalador e SHA256SUMS.txt enviados, com digests retornados pelo GitHub conferidos. Registro em `docs/evidence/sprint06/github-release.json`. Tag aponta para `f19f1ad`, fonte do pacote. Não é aprovação da V1.0: OAuth ausente, carga Google real, persistência após reinício, desinstalação e homologação Windows completa continuam pendentes. Nenhuma baseline aprovada, autostart ativado ou arquivo Google alterado.

Próxima ação: disponibilizar a configuração OAuth da aplicação por caminho local seguro e concluir a homologação pendente. Preservar o pacote instalado; não reinstalar nem reconstruir sem necessidade.
