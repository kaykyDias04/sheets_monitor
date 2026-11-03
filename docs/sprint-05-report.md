# Sprint 05 — monitoramento contínuo

Estado: implementação e validação em andamento. Este documento ainda não declara encerramento nem entrega visual de notificações Windows.

Autorização exclusiva: SM-022 a SM-025, conforme [anexo preservado](sprint05.original.md). Branch `codex/sprint-05-continuous-monitoring`, base `4041b71`. A primeira fase está no commit `dac71dc`. Nenhuma escrita Google, execução de script, alteração de credenciais ou aprovação de baseline real foi realizada pelo agente. O instalador e a campanha final de 30 planilhas pertencem à Sprint 06 e não foram executados.

## Implementação por ID

| ID | Implementação presente | Validação ainda necessária |
|---|---|---|
| SM-022 | Serviço Rust independente do WebView, fila por conta/geração, intervalo 60–300s, limite global três, exclusão por arquivo e agendas persistidas | Gates finais, build e prova nativa oculta |
| SM-023 | Outbox atômico, tentativas duráveis, preferência explícita, payload nativo, callback por ID opaco e histórico paginado | Emissão Windows autorizada e confirmação visual humana |
| SM-024 | Bandeja, instância única, X oculta com fallback acessível, saída limitada e opt-in HKCU | Inspeção Windows real; habilitação/login autostart não inferidos por mocks |
| SM-025 | Backoff/jitter, Retry-After integral persistido, cancelamento, isolamento de conta, recuperação e eventos operacionais deduplicados | Suíte final e prova de execução nativa; falhas Google destrutivas continuam sintéticas |

## Arquivos e decisões

Backend: `src-tauri/src/monitoring/`, `database/monitoring_store.rs`, `migrations/0005_continuous_monitoring.sql`, `notifications/service.rs`, `notifications/native.rs`, `desktop.rs`, wiring em `app.rs`/`lib.rs` e adaptações mínimas do coletor, transporte e persistência de integridade. Migrações 0001–0004 preservadas. Schema 5, cinco tabelas novas, 29 tabelas de produto.

Interface: `src/pages/MonitoringPanel.tsx`, `MonitoringWorkspace.tsx`, controles operacionais e preferências, contratos em `src/types/monitoring.ts` e `src/services/monitoring.ts`. Atualização por eventos Tauri, sem timer de consultas no React. Solicitações individuais e globais entram na fila Rust. Registro Windows é observado e alterado separadamente das preferências Google.

Teste nativo: recurso opcional `native-probe`, desligado no produto, exemplo `src-tauri/examples/native_probe.rs` e suporte `src-tauri/src/native_probe.rs`. Identificador e banco isolados, relógio/executor injetados, autenticação sintética sem rede. Esse recurso não abre dados/cofre de produção nem oferece comandos de diagnóstico no produto. Toast de teste fica desligado por padrão e exige autorização/flag explícitas.

Decisão em [ADR 0010](adr/0010-continuous-monitoring-delivery.md). Plugins estáveis oficiais pesquisados e fontes em [pesquisa nativa](sprint05-native-research.md). Registro autostart exclusivamente HKCU evita ampliação de privilégio e preserva entradas conflitantes.

## Testes efetivamente observados até este registro

- Baseline antes da implementação: 188 unitários Rust e duas integrações; 42 React aprovados.
- Persistência: 71 testes DB focados, seguidos por 24 testes atualizados do store, duas integrações de disco e sete do dispatcher com sink simulado. [Comandos e resultados](evidence/sprint05/database-tests.md).
- Scheduler: dez testes iniciais e doze após a correção da fila por geração; relógio e executor simulados, workers reais. [Log atual](evidence/sprint05/scheduler-gate2-tests.log).
- Cancelamento: cinco testes focados em [cancellation-tests.log](evidence/sprint05/cancellation-tests.log).
- Adaptadores: quatro testes de autostart simulado e um de payload, sem enviar toast ou gravar registro. [Resultados](evidence/sprint05/native-adapters-tests.md).
- Interface: 47 testes completos aprovados antes da adição da jornada individual; 15 jornadas App aprovadas após fila individual e correção de atualização concorrente. Lint e TypeScript aprovados naquele estado; verificações finais serão registradas separadamente.
- Build frontend aprovado. Check Rust MSVC e check do exemplo nativo aprovados; isso ainda não é comprovação de execução Windows ou build final validado.

Falhas intermediárias são preservadas em `docs/evidence/sprint05/`; incluem seletores antigos após mudança de rótulos e cleanup de testes React. Gate 1 corrigiu perda do embargo Retry-After, finalização operacional de sessão antiga, falha de criação de worker e testes do dispatcher. Gate 2 corrigiu pedidos manuais perdidos na troca de conta e atualização do dashboard recebida durante outra operação. Não se inferem aprovação dos gates finais ou CI hospedado neste registro.

## Limites e intervenção humana

`DispatchSubmitted` comprova submissão ao plugin, sem confirmação de aceitação, exibição ou leitura pelo Windows. Não incomodar e preferências do sistema podem impedir exibição. Crash entre submissão e confirmação SQLite pode causar repetição visual; cada evento distinto continua representado no histórico. Preferência desligada suprime terminalmente, sem backfill ao ligar. TTL efetivo também expira pendências sem chamá-las de entregues.

O estado real atual das proteções será observado sem alterar escolhas do usuário. Baselines pendentes exigem revisão humana; nenhuma seleção antiga será restaurada automaticamente para habilitar monitoramento. Regras e recursos inacessíveis não são substituídos por fixtures na interface operacional. Cenários sintéticos e integração Google real terão registros separados.

Faltam a validação nativa final, a confirmação humana de um alerta autorizado e a reconciliação de evidências/checks para encerrar. Autostart no login, identidade final instalada e carga integrada de 30 arquivos ficam para homologação pertinente; o agente não habilita inicialização automaticamente. Não há remoto GitHub autorizado: commits e rascunho PR serão locais.
