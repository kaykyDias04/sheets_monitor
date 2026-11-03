# Sprint 05 — monitoramento contínuo

Estado: **Sprint 05 concluída tecnicamente**, com os três gates aprovados. O alerta corrigido foi exibido no Windows e confirmado pelo usuário em 03/11/2026. A falha da primeira tentativa permanece registrada. [Gate final](evidence/sprint05/gate3.md).

Autorização exclusiva: SM-022 a SM-025, conforme [anexo preservado](sprint05.original.md). Branch `codex/sprint-05-continuous-monitoring`, base `4041b71`; fases em `dac71dc` e `23e781e`. Nenhuma escrita Google, execução de script, alteração de credenciais ou aprovação de baseline real foi realizada pelo agente. O instalador e a campanha final de 30 planilhas pertencem à Sprint 06 e não foram executados. O teste unitário sintético de escala herdado da Sprint 03 foi preservado e reexecutado na suíte; não equivale à campanha Google de 30 arquivos.

## Implementação por ID

| ID | Implementação presente | Validação ainda necessária |
|---|---|---|
| SM-022 | Serviço Rust independente do WebView, fila por conta/geração, intervalo 60–300s, limite global três, exclusão por arquivo e agendas persistidas | Aprovado localmente e em execução Windows oculta com recursos sintéticos; proteções reais atuais pendentes |
| SM-023 | Outbox atômico, tentativas duráveis, preferência explícita, WinRT com erros observáveis, callback por ID opaco e histórico paginado | Alerta sintético corrigido exibido e confirmado pelo usuário; identidade própria instalada e clique real separados |
| SM-024 | Bandeja, instância única, X oculta com fallback acessível, saída limitada e opt-in HKCU | Ciclo de vida Windows aprovado; clique físico no menu e habilitação/login autostart não verificados |
| SM-025 | Backoff/jitter, Retry-After integral persistido, cancelamento, isolamento de conta, recuperação e eventos operacionais deduplicados | Suíte completa e retomada nativa aprovadas; falhas Google destrutivas continuam sintéticas |

## Arquivos e decisões

Backend: `src-tauri/src/monitoring/`, `database/monitoring_store.rs`, `migrations/0005_continuous_monitoring.sql`, `notifications/service.rs`, `notifications/native.rs`, `desktop.rs`, wiring em `app.rs`/`lib.rs` e adaptações mínimas do coletor, transporte e persistência de integridade. Migrações 0001–0004 preservadas. Schema 5, cinco tabelas novas, 29 tabelas de produto.

Interface: `src/pages/MonitoringPanel.tsx`, `MonitoringWorkspace.tsx`, controles operacionais e preferências, contratos em `src/types/monitoring.ts` e `src/services/monitoring.ts`. Atualização por eventos Tauri, sem timer de consultas no React. Solicitações individuais e globais entram na fila Rust. Registro Windows é observado e alterado separadamente das preferências Google. Disponibilidade de notificações tem consulta somente leitura; reenvio de alertas bloqueados exige ação explícita e preserva supressão/TTL.

Teste nativo: recurso opcional `native-probe`, desligado no produto, exemplo `src-tauri/examples/native_probe.rs` e suporte `src-tauri/src/native_probe.rs`. Identificador e banco isolados, relógio/executor injetados, autenticação sintética sem rede. Esse recurso não abre dados/cofre de produção nem oferece comandos de diagnóstico no produto. Toast de teste fica desligado por padrão e exige autorização/flag explícitas.

Decisões em [ADR 0010](adr/0010-continuous-monitoring-delivery.md) e [ADR 0011](adr/0011-windows-notification-errors.md). Fontes em [pesquisa nativa](sprint05-native-research.md). O plugin inicial de notificações foi substituído pela API oficial Windows após a falha visual. Instância única permanece em Tauri; autostart exclusivamente HKCU preserva entradas conflitantes sem elevação.

## Testes efetivamente executados

- Baseline antes da implementação: 188 unitários Rust e duas integrações; 42 React aprovados.
- Persistência: 71 testes DB focados, seguidos por 24 testes atualizados do store, duas integrações de disco e sete do dispatcher com sink simulado. [Comandos e resultados](evidence/sprint05/database-tests.md).
- Scheduler: dez testes iniciais e doze após a correção da fila por geração; relógio e executor simulados, workers reais. [Log atual](evidence/sprint05/scheduler-gate2-tests.log).
- Cancelamento: cinco testes focados em [cancellation-tests.log](evidence/sprint05/cancellation-tests.log).
- Adaptadores: quatro testes de autostart simulado e um de payload, sem enviar toast ou gravar registro. [Resultados](evidence/sprint05/native-adapters-tests.md).
- Suíte completa final após a correção Windows: **257 unitários Rust aprovados, duas integrações aprovadas, um teste de cofre ignorado por padrão**. O cofre permanece inalterado e suas evidências anteriores continuam históricas. [Log MSVC](evidence/sprint05/winrt-final/rust-test.log). A rodada anterior de 242 testes permanece como evidência intermediária.
- Interface: **50 testes aprovados** em dez arquivos, incluindo fila individual, clique de notificação com sessão corrente, descarte de conta antiga, atualização concorrente, bloqueio Windows e reenvio explícito. [Log](evidence/sprint05/winrt-final-frontend.log). Lint, TypeScript, formato e Clippy aprovados no estado corrigido. [Clippy final](evidence/sprint05/winrt-final/rust-clippy.log).
- Build frontend e build Tauri MSVC com assets incorporados aprovados. O binário Windows foi executado, sem Vite ou servidor hospedado. O build definitivo da correção WinRT está em `winrt-final/`; [hash e metadados](evidence/sprint05/build-validation.json) identificam o executável validado. Rebuilds intermediários permanecem em `final-corrected/`.

Falhas intermediárias são preservadas em `docs/evidence/sprint05/`; incluem seletores antigos após mudança de rótulos, cleanup de testes React e Clippy por posição do módulo de teste. Gate 1 corrigiu perda do embargo Retry-After, finalização operacional de sessão antiga, falha de criação de worker e testes do dispatcher. Gate 2 corrigiu pedidos manuais perdidos na troca de conta e atualização do dashboard recebida durante outra operação. CI hospedado e PR remoto não foram executados.

## Execução Windows e cenários

O [probe nativo](evidence/sprint05/native-probe-validated.json) executou em processo Windows real, com banco temporário e relógio simulado: três verificações completas, uma com janela oculta, intervalos exatos de 60.000/300.000ms, pausa, retomada, coordenador compartilhado, janela restaurada e saída limitada. Zero chamadas Google, zero toasts, zero acesso ao banco/cofre de produção. Aprovações de baseline ocorreram somente nas fixtures e usam origem sintética, autorizada apenas em testes ou no recurso opt-in do probe; o build padrão e o IPC de produção não recebem essa autorização.

Falhas do exemplo foram corrigidas e preservadas: ausência de manifesto Common Controls 6 causou erro de carregamento `0xC0000139`; o manifesto exclusivo do exemplo resolveu a inicialização. Em seguida, a guarda de origem sintética precisava admitir o recurso de teste opt-in, sem liberar esse caminho no produto. Avisos não fatais de ícone do executável de exemplo e limpeza Chromium constam nas notas. Não se inferiu sucesso apenas pela compilação.

A [validação do aplicativo real](evidence/sprint05/native-qa.json) confirmou conta conectada, schema 5/29 tabelas, quatro cadastros pendentes e zero proteções elegíveis. Início/pausa foram testados sem criar agendas ou acessar Google; o estado final voltou a pausado. Fechar ocultou a janela preservando processo/controle ativo; a segunda abertura terminou com código 0 e restaurou a mesma instância. Sair pelo botão encerrou o processo e fechou a porta de diagnóstico, sem encerramento forçado. Autostart foi apenas observado desativado.

A automação Windows não expôs a janela da barra de tarefas; não se afirma clique físico no menu da bandeja. O probe exercitou os callbacks e a saída nativa, e o aplicativo real exercitou o botão Sair pelo mesmo shell. Agent-browser apresentou timeout de CDP mesmo com expressão simples e sessão nova; a inspeção continuou pela API Windows computer-use. Não se afirma auditoria Axe nova ou screenshot público redigido por DOM. Capturas da inspeção não foram adicionadas ao Git.

Os [20 cenários exigidos](evidence/sprint05/scenario-matrix.json) apontam nomes encontrados no log final de testes e distinguem fixtures, Windows real e leituras Google históricas. Clique físico e confirmação de toast continuam explicitamente separados.

## Correção e confirmação da entrega

O usuário autorizou um alerta inicial, não exibido, e depois um único alerta corrigido. A [primeira observação negativa](evidence/sprint05/native-toast-observation.json) permanece intacta. A análise identificou a heurística de caminho/identidade do plugin e erros descartados em tarefas de envio. O adaptador passou a usar WinRT diretamente, consultar configuração sem alterá-la e registrar HRESULT/Failed por tentativa, sem thread por alerta. Conta, geração, preferência e TTL continuam protegidos. Um callback que ocorre durante Show apenas enfileira recibo, evitando reentrância do lock de autenticação.

Consulta real sem enviar outro alerta confirmou criação do notifier e erro apenas no getter opcional Setting (`native_setting_hresult_80070490`). Somente esse erro é tratado como configuração desconhecida que admite uma tentativa autorizada; valores Disabled continuam bloqueados e outros erros viram retentativa. Não se infere permissão nem exibição a partir da consulta.

Na tentativa corrigida, o usuário confirmou **“Sim, apareceu”**, com remetente Windows PowerShell informado previamente. [Confirmação humana](evidence/sprint05/native-toast-winrt-observation.json). O processo do probe recebeu um sinal Windows de interrupção/fechamento antes de gravar seu JSON final; o código de saída e a limitação estão em [registro da execução](evidence/sprint05/native-toast-winrt-run.json). A origem do sinal não foi estabelecida. Não se afirma coleta completa de callbacks ou saída limpa desse teste específico; a saída limitada já fora comprovada nos probes sem toast e no aplicativo real. Nenhum alerta adicional foi enviado para regenerar o relatório.

Seis testes do adaptador validaram identidade, HRESULT, configuração desconhecida, bloqueios conhecidos e XML hostil em memória. Cinco regressões do store e doze do dispatcher verificaram falha antes/depois de submissão, CAS da tentativa, logout, supressão e preservação de recibos após erro de banco. A submissão nativa continua separada de exibição, clique e leitura.

## Limites e intervenção humana

`DispatchSubmitted` registra a chamada nativa sem inferir exibição ou leitura. Não incomodar e preferências do sistema podem impedir exibição. Crash entre submissão e confirmação SQLite pode causar repetição visual; cada evento distinto continua representado no histórico. Preferência desligada suprime terminalmente, sem backfill ao ligar. TTL efetivo também expira pendências sem chamá-las de entregues.

O estado real atual das proteções foi observado sem alterar escolhas do usuário. Baselines pendentes exigem revisão humana; nenhuma seleção antiga foi restaurada para habilitar monitoramento. Regras e recursos inacessíveis não foram substituídos por fixtures na interface operacional. Não houve consulta contínua Google real nesta sprint, pois nenhum escopo atual estava elegível.

A confirmação visual foi obtida e o gate final aprovado. Autostart no login, identidade final instalada, clique físico no menu/alerta e carga integrada de 30 arquivos ficam para homologação pertinente; o agente não habilitou inicialização automaticamente. O console extra observado no desenvolvimento foi eliminado pelo atributo Windows GUI nos entry points do produto e do probe; PE e probe sem toast validaram o ajuste, sem alterar o backend. A tentativa final de reinspeção visual foi interrompida pelo usuário com Esc e nenhuma ação Computer Use foi realizada depois; não se afirma nova captura visual desse último lançamento. Não há remoto GitHub autorizado: commits e rascunho PR são locais. A implementação permite iniciar a Sprint 06 quando o usuário enviar seu prompt, com esses limites explícitos. Nenhuma etapa seguinte será iniciada automaticamente.
