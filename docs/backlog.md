# Backlog rastreável — especificação v1.0

IDs, títulos e aceite principal preservam o anexo. Dependências e subtarefas são detalhamento de engenharia. Status exige evidência; código arquivado não conclui tarefa. A fonte estruturada é [backlog.json](backlog.json).

Autorização vigente: **Sprint 03 concluída no aceite técnico local**, SM-012 a SM-017. Sprints 01/02 concluídas, com homologação Google real na Sprint 02. Etapas 04–06 aguardam autorização. Aprovação de baseline real permanece humana. Mutações reais continuam pendentes; ver [encerramento](evidence/sprint03/final-report.md).

## Sprint 01

| ID | Tarefa oficial | Estado | Dependências | Aceite |
|---|---|---|---|---|
| SM-001 | Verificar e preparar ambiente Windows | concluido | — | Dependências instaladas |
| SM-002 | Criar repositório e projeto Tauri | concluido | SM-001 | Aplicativo inicia |
| SM-003 | Configurar React, TypeScript e UI | concluido | SM-002 | Interface navegável |
| SM-004 | Estruturar backend Rust | concluido | SM-002 | Módulos organizados |
| SM-005 | Criar SQLite e migrações | concluido | SM-004 | Persistência testada |
| SM-006 | Configurar testes e documentação | concluido | SM-003, SM-004, SM-005 | Build e testes aprovados |

### SM-002 — subtarefas

- **SM-002.01** — Preservar protótipo e histórico sem sobrescrever (concluido).
- **SM-002.02** — Organizar governança, licença, templates e branch (concluido).
- **SM-002.03** — Abrir janela Tauri nativa com demonstração (concluido).

### SM-003 — subtarefas

- **SM-003.01** — Configurar Tailwind/shadcn, aliases e componentes (concluido).
- **SM-003.02** — Navegar em quatro telas com fixtures explícitas (concluido).

### SM-004 — subtarefas

- **SM-004.01** — Separar shell, banco e contratos futuros sem acesso Google (concluido).

### SM-005 — subtarefas

- **SM-005.01** — Criar onze entidades e ledger de migrações (concluido).
- **SM-005.02** — Testar criação/leitura, reabertura e rollback (concluido).

### SM-006 — subtarefas

- **SM-006.01** — Documentar escopo, arquitetura, riscos e aceite (concluido).
- **SM-006.02** — Executar lint, formatação, typecheck, testes e build (concluido).
- **SM-006.03** — Configurar CI Windows e registrar execução nativa (concluido).

## Sprint 02

| ID | Tarefa oficial | Estado | Dependências | Aceite |
|---|---|---|---|---|
| SM-007 | Implementar OAuth com PKCE | concluido | SM-004, SM-006 | Login Google funcional |
| SM-008 | Implementar armazenamento seguro de tokens | concluido | SM-005, SM-007 | Sessão recuperável |
| SM-009 | Integrar Sheets API | concluido | SM-007, SM-008 | Ler planilha autorizada |
| SM-010 | Integrar Drive API | concluido | SM-007, SM-008 | Ler metadados |
| SM-011 | Integrar Apps Script API | concluido | SM-007, SM-008 | Ler projeto autorizado |

### SM-007 — subtarefas

- **SM-007.01** — PKCE, callback e identidade com transporte simulado (concluido).

### SM-008 — subtarefas

- **SM-008.01** — Cofre Windows, renovacao e isolamento de conta (concluido).

### SM-009 — subtarefas

- **SM-009.01** — Metadados, abas e batchGet de valores/formulas por intervalos (concluido).

### SM-010 — subtarefas

- **SM-010.01** — Metadados Drive por fileId e diagnostico de acesso (concluido).

### SM-011 — subtarefas

- **SM-011.01** — Fontes e manifesto de IDs explicitamente autorizados, hash e associacao (concluido).

## Sprint 03

| ID | Tarefa oficial | Estado | Dependências | Aceite |
|---|---|---|---|---|
| SM-012 | Implementar baseline versionado | concluido | SM-005, SM-009, SM-010, SM-011 | Referência persistida |
| SM-013 | Detectar alterações de abas e colunas | concluido | SM-012 | Diferenças identificadas |
| SM-014 | Comparar fórmulas protegidas | concluido | SM-009, SM-012 | Falhas detectadas |
| SM-015 | Analisar dependências Apps Script | concluido | SM-011, SM-012, SM-013 | Referências verificadas |
| SM-016 | Classificar integridade | concluido | SM-013, SM-014, SM-015 | Estados corretos |
| SM-017 | Registrar mudanças de conteúdo | concluido | SM-009, SM-012, SM-016 | Eventos sem duplicação indevida |

### SM-012 — subtarefas

- **SM-012.01** — Prévia, aprovação explícita e histórico de versões (concluido).

### SM-013 — subtarefas

- **SM-013.01** — Detectar remoção/deslocamento de abas e colunas protegidas (concluido).

### SM-014 — subtarefas

- **SM-014.01** — Comparar regras de fórmulas e erros efetivos (concluido).

### SM-015 — subtarefas

- **SM-015.01** — Resolver getSheetByName/getRange/índices estáticos (concluido).
- **SM-015.02** — Sinalizar dependências dinâmicas não verificáveis (concluido).

### SM-016 — subtarefas

- **SM-016.01** — Separar integridade, mudança permitida e incerteza técnica (concluido).

### SM-017 — subtarefas

- **SM-017.01** — Persistir todas as diferenças sem duplicação indevida (concluido).

## Sprint 04

| ID | Tarefa oficial | Estado | Dependências | Aceite |
|---|---|---|---|---|
| SM-018 | Criar dashboard | planejado | SM-003, SM-016 | Indicadores funcionais |
| SM-019 | Implementar cadastro por link | planejado | SM-005, SM-010, SM-018 | Adicionar/remover arquivos |
| SM-020 | Criar seleção de abas | planejado | SM-012, SM-019 | Escopo individual configurável |
| SM-021 | Criar histórico | planejado | SM-017, SM-018 | Eventos pesquisáveis |

### SM-019 — subtarefas

- **SM-019.01** — Editar, remover, pausar e impor capacidade de trinta arquivos (planejado).

### SM-020 — subtarefas

- **SM-020.01** — Escopo por arquivo/abas e efeito sobre baseline aprovado (planejado).

## Sprint 05

| ID | Tarefa oficial | Estado | Dependências | Aceite |
|---|---|---|---|---|
| SM-022 | Criar agendador | planejado | SM-016, SM-017, SM-020 | Consultas automáticas |
| SM-023 | Criar notificações nativas | planejado | SM-017, SM-022 | Alertas no Windows |
| SM-024 | Implementar bandeja do sistema | planejado | SM-022, SM-023 | Monitoramento minimizado |
| SM-025 | Implementar recuperação de falhas | planejado | SM-009, SM-010, SM-011, SM-022 | Tratamento de internet e cotas |

### SM-022 — subtarefas

- **SM-022.01** — Intervalo 60–300s, até três coletas e aviso de frequência inviável (planejado).

### SM-023 — subtarefas

- **SM-023.01** — Outbox durável e consolidação sem perda de eventos (planejado).

### SM-024 — subtarefas

- **SM-024.01** — Minimização/bandeja e autostart opcional explícito (planejado).

### SM-025 — subtarefas

- **SM-025.01** — Rede/cotas/permissões e recuperação com estado Indeterminado (planejado).

## Sprint 06

| ID | Tarefa oficial | Estado | Dependências | Aceite |
|---|---|---|---|---|
| SM-026 | Testes integrados | planejado | SM-006, SM-008, SM-016, SM-017, SM-020, SM-023, SM-024, SM-025 | Cenários críticos aprovados |
| SM-027 | Teste de carga com 30 planilhas | planejado | SM-022, SM-026 | Estabilidade comprovada |
| SM-028 | Gerar instalador | planejado | SM-026, SM-027 | Instalação Windows validada |

### SM-026 — subtarefas

- **SM-026.01** — Matriz de cenários reais autorizados e fixtures separadas (planejado).

### SM-027 — subtarefas

- **SM-027.01** — Medir memória, duração e quotas com trinta arquivos (planejado).

### SM-028 — subtarefas

- **SM-028.01** — Auditar licenças/avisos e dependências do bundle (planejado).
- **SM-028.02** — Validar instalação/desinstalação Windows x64 e manual (planejado).

Estimativa original: 25–41 dias úteis, apenas planejamento. Credenciais, consentimento e homologação final não são presumidos. Consulte [critérios de aceite](acceptance.md) e [progresso](progress.md).

## Homologação Sprint 02 concluída

SM-007.02 a SM-011.02: concluídos por testes reais com conta e recursos autorizados. Evidência sanitizada em [google-live](evidence/sprint02/google-live.json). Segredos, identidade, IDs e conteúdos não foram registrados. Aguardar Prompt 03 sem antecipar a próxima sprint.
