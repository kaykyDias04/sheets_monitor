# Sprint 06 — prontidão inicial (Gate 1)

Inspeção em 03/11/2026, base `b2c68ce0791aa8acdcc860f433dd2ed6a61849da`, branch `codex/sprint-06-homologation`. Este relatório é uma auditoria estática inicial, não a aprovação da V1.0. Prompt autorizado preservado em `docs/sprint06.original.md`.

## Estado observado

- Sprints 01–05 possuem implementação ativa e evidências históricas. SM-022–SM-025 estão concluídos tecnicamente; SM-026–SM-028 não ganham aprovação por herança.
- Alterações preexistentes em AGENTS.md, README.md, docs/architecture.md e docs/progress.md e novos NEXT_STEPS/context-transfer foram preservados. A árvore não estava limpa; a fotografia histórica de árvore limpa não descreve o início desta sprint.
- Frontend React/TypeScript/Vite/Tailwind, backend Rust/Tauri 2, SQLite schema 5; migrações 0001–0005 presentes. `archive/prototype/` é referência fora dos imports/builds ativos.
- AuthManager implementa PKCE/loopback, identidade UserInfo, sessão no cofre Windows, refresh, logout local e revogação explícita. Leitores Google usam GET em endpoints fixos, sem escrita Sheets/Drive/Apps Script ou execução de script.
- Motor mantém snapshots, versões aprovadas, diff estrutural/fórmulas/conteúdo, análise estática, classificação e eventos. Registro por conta e revisão CAS; runtime Rust controla fila, até três trabalhos, polling, outbox e recuperação; React é cliente IPC.
- Bandeja, instância única, WinRT e autostart opt-in têm implementação. Envio aceito não comprova exibição/leitura. Aprovação de baseline exige gesto humano real no produto; origem sintética permitida apenas em teste/recurso opt-in.

## Cobertura existente e caminho executável

A contagem histórica final é 257 unitários Rust + 2 integrações e 50 React, com um teste de cofre ignorado por padrão. Novos resultados dependem exclusivamente dos logs da Sprint 06; este documento não declara a reexecução aprovada.

| Requisito | Testes/fontes executáveis existentes | Limite |
|---|---|---|
| Login/logout/refresh/expiração/consentimento | `auth::manager::tests`, `auth::protocol::tests` | Transporte e cofre simulados; cofre real tem roundtrip UUID separado |
| Cadastro/edição/remoção/pausa/capacidade | `management::tests` | Banco/conta sintéticos; registro real depende de leitura autorizada |
| Seleção de abas/regras/baseline | `management::tests`, `integrity::runtime::tests` | Aprovações sintéticas, sem autorização de aprovação real pelo agente |
| Colunas/fórmulas/abas/linhas/Apps Script | `integrity::engine_tests`, `script_tests`, `google::tests`, `collector::tests` | Algoritmos locais; não prova alteração destrutiva Google real |
| Estados/histórico/isolamento | `management::tests`, `integrity_store_tests`, testes React | Cobertura de estado em fixtures, sem taxa percentual de cobertura instrumentada |
| Fila/limite três/intervalo/retomada | `monitoring::service_tests`, `work::tests`, `policy::tests` | Relógio/executor simulados; precisa campanha integrada 5/15/30 |
| Outbox/deduplicação/falha de entrega | `notifications::service_tests`, `monitoring_store_tests` | Sink simulado; toast físico instalado continua separado |
| Migrações/reabertura/falha | `database::migrations::tests`, `tests/database_persistence.rs` | Bases temporárias; preservar bases reais e hashes históricos |
| Minimização/ciclo de vida | `native-probe` e histórico Sprint 05 | Nova prova de produção instalada ainda necessária |

Comandos baseline: `npm run lint`, `npm run format:check`, `npm run typecheck`, `npm test`, `npm run build`; `scripts/check-rust.ps1 -Task format-check|test|clippy|build -EvidenceDirectory docs/evidence/sprint06/<rodada>`. O verificador atual contém bloqueio histórico da Sprint 06 e destino padrão Sprint 05: precisa evolução explícita antes de executá-lo para não regravar evidências antigas.

## Requisitos de build e configuração de produção

Windows 11 x64, MSVC/Windows SDK, Rust >=1.90 e Node/npm conforme lockfile existente. Toolchains já foram validados historicamente, sem inferir estado atual. Instalação exige WebView2; execução sem SDK/npm/Rust deve ser provada em máquina limpa ou ambiente isolado disponível.

Manifestos iniciais ainda usam versão `0.1.0`; `tauri.conf.json` tem `bundle.active=false`, alvo NSIS e identidade `com.sheetsmonitor.desktop`. Não existe instalador V1.0 validado nesta auditoria. A provisão OAuth opcional vem de `SHEETS_MONITOR_OAUTH_CLIENT_ID`/`SHEETS_MONITOR_OAUTH_DESKTOP_SECRET` em build; não está comprovada para novo perfil/artefato distribuível. Desktop client é público, PKCE obrigatório; nunca incorporar refresh/access token.

A pasta de instalação não isola o perfil: `app.rs` abre `app_data_dir()/foundation-v1.db` e `WindowsVault::production()` usa namespace fixo. Instalar em outro diretório e iniciar o executável pode acessar estado real existente. Para prova sem esse acesso, usar usuário Windows/VM isolados ou mecanismo explicitamente revisado; não copiar cofre/banco reais para evidências.

## Lacunas obrigatórias antes do parecer final

1. Registrar baseline atual e testes integrados cruzando runtime/banco/serviço, inclusive reinício/falhas/duplicação.
2. Campanha progressiva 5/15/30 com métricas de tempo, CPU/memória, SQLite e simultaneidade; simulados não comprovam cotas/vazão real. Meta 24h precisa execução ou pendência explícita.
3. Homologação Google do candidato final no ambiente autorizado. Histórico registra quatro cadastros pendentes/zero elegíveis; não restaurar seleções nem aprovar referências pelo agente para obter carga.
4. Notificação com identidade instalada e confirmação visual, sem enviar novo toast sem autorização específica/preferência. Histórico WinRT de desenvolvimento não substitui essa prova.
5. Build produção, avisos/licenças transitivos, hash, instalador x64, instalação/desinstalação reais e operação sem ferramentas de desenvolvimento.
6. OAuth de distribuição, assinatura/reputação do binário, documentação e homologação humana final; sem publicação/release/distribuição externa.

## Parecer inicial

Há base técnica para executar a fase local de homologação. **V1.0 ainda não aprovada**. Lacunas acima não alteram critérios de aceite; o relatório final deve classificá-la conforme evidências, inclusive BLOQUEADA se requisitos essenciais permanecerem não comprovados. A ausência de prova não é um defeito confirmado de implementação.
