# Contratos IPC — Sprint 02

O renderer recebe DTOs tipados; endpoints, SQL, caminho do banco e tokens não são argumentos de IPC. A janela principal conserva apenas permissões locais Tauri. As operações de rede/cofre executam em `spawn_blocking`, fora da thread de interface.

| Comando | Entrada | Saída |
|---|---|---|
| `foundation_get_dashboard` | Nenhuma | Fixtures com `is_demo: true`, preferências e versão do banco |
| `foundation_save_settings` | `settings` | Confirmação local; sem scheduler |
| `integration_get_state` | Nenhuma | Configuração pública, conta, escopos, expiração conhecida e geração |
| `integration_configure` | `input`: client ID Desktop, secret opcional, opção Drive | Estado; encerra sessão anterior e exige novo consentimento |
| `integration_connect` | Nenhuma | Estado após PKCE, token exchange e UserInfo |
| `integration_disconnect` | Nenhuma | Estado sem sessão local; não revoga Google |
| `integration_revoke` | `confirmed: true` | Estado após POST explícito de revogação |
| `integration_test_sheet` | `input`: URL/ID, lista de intervalos A1 | Resultados independentes de metadados, valores/fórmulas e Drive |
| `integration_test_script` | `input`: script ID, spreadsheet ID opcional explícito | Projeto, arquivos tipados e hash |

`IntegrationState` contém `generation`, `configured`, `client_id`, `auth` e `drive_metadata_opt_in`. O aceite de engenharia fica nas evidências versionadas, separado do estado de uma conta em execução. O antigo campo fixo `real_validation: pending` foi removido após a homologação real, pois não descrevia a sessão do usuário.

`auth` contém status `unconfigured`, `disconnected`, `connected` ou `reconnect_required`, identidade não secreta, escopos efetivamente concedidos e `expires_at` em segundos Unix quando conhecido. Recuperar estado não faz login nem abre navegador. A primeira consulta renova a sessão protegida recuperada, se necessário.

Resultados têm `generation` e `account_subject`. Cada serviço retorna `status` (`success`, `error`, `not_authorized`, `not_requested`), `data` ou erro fechado (`code`, `message`, `retryable`). Não retornar corpo bruto de erro do Google. O React mapeia categorias conhecidas e ignora mensagens arbitrárias. Não há fallback de fixtures para chamadas reais.

Geração protege gravações e montagem da resposta no backend. O renderer também verifica geração/conta e apaga resultados após mudança de sessão. Um 401 permite uma única renovação e repetição da leitura; o transporte repete apenas GETs transitórios dentro de limites. Nenhum comando executa scripts, aprova baseline ou cadastra monitor.

## Sprint 03 — operações de integridade

Todos os comandos passam pelo backend autenticado e retornam `EngineEnvelope<T>` com geração, identidade da conta e dados. O renderer rejeita mudança entre sessão inicial/final ou envelope divergente. Dados reais são apenas texto na UI; nenhuma lógica de classificação é reimplementada no React.

| Comando | Entrada | Resultado |
|---|---|---|
| integrity_propose_scope | url | Proposta editável e inventário de abas; não aprova |
| integrity_capture_snapshot | input: url, scope | Snapshot privado, fingerprint, geração da prévia e resumo da referência ativa |
| integrity_create_baseline | input: snapshot_id, snapshot_fingerprint, expected_generation, expected_version, confirmed | Nova versão aprovada explicitamente |
| integrity_list_baselines | input: spreadsheet_id, scope_hash; offset opcional | Até5 resumos por página |
| integrity_get_active_baseline | input: spreadsheet_id, scope_hash | Referência ativa correspondente à conta/escopo |
| integrity_compare_snapshot | snapshotId | Comparação sem gravar relatório/eventos |
| integrity_validate_integrity | snapshotId | Relatório e eventos persistidos transacionalmente |
| integrity_analyze_script_dependencies | snapshotId | Análises e hashes capturados; sem fonte bruta |
| integrity_list_integrity_events | input: spreadsheet_id, scope_hash | Até200 eventos recentes |
| integrity_get_integrity_report | input: spreadsheet_id, scope_hash | Último relatório do escopo |
| integrity_prune_history | input: spreadsheet_id, scope_hash | Quantidade de registros removidos por retenção configurada |

Argumentos top-level snapshotId usam a convenção camelCase do Tauri; campos dentro de input seguem serde snake_case. O IPC não aceita snapshot arbitrário, SQL, caminho de arquivo, URL de API ou origem sintética de aprovação. Captura e proposta possuem limite de3 operações concorrentes. Erros são categorias fechadas, sem conteúdo bruto do provedor.

Aprovação exige confirmação explícita e vincula a geração da prévia à sessão do backend antes de gravar, além de conta, ID, fingerprint e versão esperada. O mutex protege o commit local e não fica preso à rede. Leituras longas podem terminar após troca de sessão, mas seus resultados antigos não são persistidos ou aplicados à nova conta.

Os DTOs de snapshot schema1 e escopo schema1 são versionados. Conteúdo mínimo, incerteza, retenção e limites estão em [motor](integrity-engine.md), [baselines](baselines.md), [fórmulas](formula-analysis.md) e [scripts](apps-script-dependencies.md). Não há comando para alterar o Google ou iniciar monitoramento contínuo.
