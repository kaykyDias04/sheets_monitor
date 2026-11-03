# Plano de integração e carga — Sprint 06

Estado: proposta de evidência após inspeção estática; nenhum cenário desta campanha executado por este documento. Base de inspeção: b2c68ce, branch codex/sprint-06-homologation; documentos locais preexistentes preservados. Escopo e critérios: docs/sprint06.original.md, SM-026/SM-027. Google permanece somente leitura; referência real exige humano. Este plano não autoriza efeitos externos.

## Cobertura existente e lacuna

- src-tauri/src/integrity/scale_tests.rs: thirty_workbooks_with_one_thousand_records_have_bounded_payloads_and_precise_differences mede evaluate() em 30 arquivos sequenciais de 1.000 registros. Não inicia fila, persiste SQLite nem mede APIs/notificação.
- monitoring/service_tests.rs: relógio/execução simulados, workers reais, admissão global três, backoff/rede/Retry-After, logout e retomada. Executor retorna relatórios sem persistência de integridade. A suíte prova políticas específicas, não campanha integrada de 5/15/30.
- native_probe.rs: prova isolada do processo Windows, janela oculta, fixture de um arquivo, captura/evaluate/persistência sob lease e desligamento limitado. Não abre cofre/banco de produção nem Google. Não repetir --test-toast sem autorização específica.
- integrity/runtime_tests.rs e database/*_tests.rs cobrem confirmação/fingerprint/conta, baselines e transações. notifications/service.rs tem testes de dispatcher com sink simulado. Manter esses testes e acrescentar travessia conjunta.

## Implementação mínima proposta

Criar src-tauri/src/monitoring/homologation_tests.rs, incluído somente por #[cfg(test)] em monitoring/service.rs. Esse módulo filho acessa tick(), State e start_internal() já existentes sem ampliar superfície do produto. Não mudar visibilidade de API, adicionar comando IPC nem habilitar native-probe no bundle. Cargo test permite filtrar nome sprint06 e --nocapture para logs estruturados.

Fixture: tempfile::TempDir, Database::open de caminho absoluto, AuthManager com CredentialVault em memória e OAuthTransport que falha se houver chamada de rede. Identidade e IDs exclusivos sintéticos. Registrar arquivos com register_operational_spreadsheet, set_registered_tabs/set_registered_rules e aprovar somente fixtures com origem synthetic:test. Usar scope.history_retention_days=90; preferências padrão de 60s e autostart falso. Nomes de arquivos nunca reais.

Executor: implementar CheckExecutor no teste, usando IntegrationRuntime.database_path e fixtures::snapshot(). Recuperar baseline/observation do DB, aplicar mutação determinística por arquivo/ciclo, salvar snapshot, evaluate(), fixar observation_revision, previous_snapshot_id e detected_at, obter work::current_lease e persist_monitored_integrity_report. Contador RAII de atividade para máximo global observado, contador por arquivo contra concorrência duplicada; checkpoint antes de gravar. Retornar EngineEnvelope/ValidationResult como no native_probe.rs. Eventos e outbox precisam surgir pela transação real, não inserts de teste artificiais.

Notificações: NotificationService::start(runtime.clone(), Arc<RecordingSink>, callback), onde RecordingSink registra instante/ID opaco e retorna DispatchSubmitted. Habilitar preferência somente no DB sintético. Assim medir latência até submissão simulada e afirmar explicitamente que não mede WinRT, exibição ou leitura. A API nativa jamais deve ser chamada. Para TTL do dispatcher, manter captured_at/detected_at/created_ms próximos de UTC real; relógio acelerado do scheduler não pode expirar artificialmente a fila. Alternativa: exercitar claim_notification/finish_notification_attempt com o mesmo relógio injetado e relatar esse caminho como store+sink, sem alegar daemon do dispatcher.

## Campanha progressiva 5, 15, 30

Para cada tamanho, banco temporário novo e captura de métricas antes/depois:

1. Baseline inicial e primeiro ciclo: todos Online, zero diferenças de conteúdo inesperadas. Observar ordem escalonada 60s sem executar 60s de espera real. Não confundir relógio lógico com tempo real.
2. Mudanças simultâneas: uma edição permitida de valor por arquivo; até três workers reais ativos e exclusão por arquivo. Todos Online, número esperado de diferenças/eventos, outbox transacional e submissões simuladas distintas.
3. Repetição sem mudança: zero novos eventos e zero novas submissões para o mesmo incidente/conteúdo. Reabrir DB e conferir persistência de baselines e histórico.
4. Dano crítico: grupos de arquivos com fórmula ausente/erro efetivo, coluna removida/reordenada, aba removida, hash Apps Script alterado ou dependência inválida. Usar expectativas existentes de engine_tests, preservando ambiguidades como Indeterminado. Recuperar e repetir para validar lifecycle sem spam.
5. Falhas: injetar network e rate_limited com work::defer_retry(Duration::from_secs(3600)); não gerar quota real. Estado operacional Indeterminado, prazo respeitado inclusive pedidos manuais/restart. Depois recuperar e conferir um evento de recuperação por incidente.
6. Pausa e restart: desligar serviço, Database::open no mesmo arquivo, comparar baseline ativo, iniciar coordenador novo e validar que não há replay ilimitado nem perda de eventos. Autostart permanece intocado.
7. Preferências: 60 e 300s; histórico padrão 90 dias. Uma execução propositalmente acima do intervalo deve produzir warning interval_capacity_exceeded observado via service.status(), nunca apenas verificar string no fonte.

Para coordenador determinístico, tick() pode ser chamado diretamente e workers drenados por predicado com timeout limitado; avançar Clock somente depois de workers drenarem. Para saturação simultânea, enfileirar request_check_for de todos os arquivos e liberar executor por barreira controlada; máximo observado deve atingir três e nunca excedê-lo. O avanço inicial de schedules deve respeitar stagger_ms; pular mais de 120s pode disparar resumed_after_gap/replan e invalidar a intenção do ciclo.

## Métricas e interpretação

| Medida | Fonte proposta | Limite da conclusão |
|---|---|---|
| Tempo de ciclo e cada verificação | Instant real e datas do scheduler | Separar wall_elapsed_ms de logical_elapsed_ms; CPU local, sem rede |
| Latência de detecção | instante real da mutação disponibilizada até evento persistido | Fixture entra no executor, não propagação Google |
| Latência de notificação | persistência do evento até RecordingSink | Submissão simulada; não toast mostrado |
| Concorrência e exclusão | contadores RAII no executor e service.status | Caminho real de fila/admissão para banco isolado |
| CPU | Get-Process CPU antes/depois no processo de teste isolado | CPU do processo, sem confundir com porcentagem instantânea |
| Memória | PowerShell sampler WorkingSet64/PrivateMemorySize64/PeakWorkingSet64 do PID | Amostragem pode perder picos; indicar período e usar pico do SO |
| SQLite | tamanho DB + WAL + SHM antes/depois, checkpoint controlado e contagens via API | Incluir WAL; DB isolado, não disco de produção |
| Eventos/retries/dedup | integrity_events, monitoring_operational_history, notification_history | Asserções de cardinalidade por fase; não só log visual |
| Quotas | transport/executor counters e embargo Retry-After | Quotas reais não medidas em campanha sem Google |

Um runner PowerShell deve iniciar somente o binário de testes exato com filtro sprint06, amostrar esse PID, salvar stdout/stderr/log e resumo JSON exclusivamente docs/evidence/sprint06. A saída não deve incluir caminho de perfil privado, IDs reais, fórmulas/código real. Usar dados sintéticos explícitos. Não medir todos os testes concorrentes como se fossem a campanha.

## Google real e ensaio prolongado

Campanha sintética não satisfaz por si só vazão/quota Google com 30 arquivos. Histórico descreve quatro cadastros pendentes e nenhuma referência elegível. Antes do gate real, reunir um checkpoint: 30 recursos disponíveis/autorizados, escopos de leitura concedidos, baselines revisadas/aprovadas pelo usuário e permissões de notificação quando necessárias. Não cadastrar/arbitrar arquivos reais nem aprovar referências para atingir a meta.

Sem esses recursos, entregar métricas locais e marcar a campanha Google de 5/15/30 como não executada. Meta de 24h: executável/runner opt-in com duração e limites explícitos pode ser preparado; 1.440 minutos acelerados não são 24h reais. Só declarar ensaio prolongado após tempo de parede e logs concluídos, sem manter chamadas Google/alertas reais em segundo plano silenciosamente. O relatório final deve manter requisito essencial não comprovado como BLOQUEADA, conforme o prompt, até a evidência ou decisão humana correspondente.

## Gate de revisão

Antes de implementar: confirmar isolamento, compatibilidade temporal do dispatcher e orçamento do ensaio com revisor. Depois: conferir execução efetiva dos três tamanhos, logs imutáveis, asserções de estado e métricas, ausência de chamadas remotas/toasts/registro, e distinção entre provas novas, históricas e pendentes. Não marcar SM-027 concluído somente por campanha local se o aceite integral continuar sem prova.
