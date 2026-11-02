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

`IntegrationState` contém `generation`, `configured`, `client_id`, `auth`, `drive_metadata_opt_in` e `real_validation: pending`. Esse último campo representa o gate externo desta entrega; sucesso de uma consulta pontual não homologa a sprint inteira.

`auth` contém status `unconfigured`, `disconnected`, `connected` ou `reconnect_required`, identidade não secreta, escopos efetivamente concedidos e `expires_at` em segundos Unix quando conhecido. Recuperar estado não faz login nem abre navegador. A primeira consulta renova a sessão protegida recuperada, se necessário.

Resultados têm `generation` e `account_subject`. Cada serviço retorna `status` (`success`, `error`, `not_authorized`, `not_requested`), `data` ou erro fechado (`code`, `message`, `retryable`). Não retornar corpo bruto de erro do Google. O React mapeia categorias conhecidas e ignora mensagens arbitrárias. Não há fallback de fixtures para chamadas reais.

Geração protege gravações e montagem da resposta no backend. O renderer também verifica geração/conta e apaga resultados após mudança de sessão. Um 401 permite uma única renovação e repetição da leitura; o transporte repete apenas GETs transitórios dentro de limites. Nenhum comando executa scripts, aprova baseline ou cadastra monitor.
