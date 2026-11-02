# Leitores Google — Sprint 02

A implementação ativa fica em `src-tauri/src/google/`; o runtime manual em `src-tauri/src/integration.rs`. Todas as consultas de recursos usam GET HTTPS com origem/caminho fixos e redirects desabilitados. Autenticação é separada em `auth/`.

| Serviço | Operação | Dados retornados |
|---|---|---|
| Sheets v4 | `spreadsheets.get` | Título, locale, timezone e todas as propriedades de abas selecionadas por fields |
| Sheets v4 | `spreadsheets.values.batchGet` duas vezes | Intervalos calculados `UNFORMATTED_VALUE` e fórmulas `FORMULA`, dimensão ROWS, datas seriais |
| Drive v3 | `files.get` | Nome, MIME, modifiedTime, trashed, versão e IDs relevantes; supportsAllDrives |
| Apps Script v1 | `projects.get` e `projects.getContent` | Metadados e arquivos SERVER_JS, HTML e JSON, com fonte textual e hash SHA-256 |

URLs aceitas: `https://docs.google.com/spreadsheets/d/ID/edit`, formas view/preview e `/spreadsheets/u/N/d/ID`. O backend também aceita ID puro. Rejeita hosts externos, HTTP, credenciais, traversal, barras invertidas e IDs com caracteres fora de letras ASCII, números, `_` e `-` (máximo 256 bytes). Não navega para o link fornecido.

Até 20 intervalos por consulta, 256 bytes cada e 3000 bytes totais. Resposta máxima 8 MiB; timeout de conexão 10s e de requisição 30s. Cada operação de provedor tem orçamento de 60s, até três tentativas GET para rede, timeout, 429, 5xx e 403 com motivo explícito de cota. Retry-After limitado a 10s. Uma consulta de tela combina operações sequenciais; não há polling. HTTP400/401/403 de permissão/404 não entram nas retentativas do transporte. Um 401 pode renovar a sessão e repetir uma leitura uma vez no runtime.

Intervalos vazios são válidos. Respostas malformadas, IDs divergentes, abas duplicadas e células compostas são rejeitadas. Os dois batchGets e as três APIs não compõem uma transação Google: uma edição entre chamadas pode produzir leituras de momentos diferentes. Não declarar snapshot atômico ou monitoramento completo.

Drive é opcional, com `drive.metadata.readonly`, conforme [ADR 0007](adr/0007-google-scopes-and-desktop-auth.md). `modifiedTime` e versão não informam todas as alterações de células nem comprovam integridade. Recursos removidos/inacessíveis resultam em diagnóstico sem conclusão Online/Offline.

Apps Script exige script ID conhecido e autorizado, podendo requerer permissão adicional no painel do serviço. A associação à planilha só é persistida quando explicitamente solicitada e quando a mesma conta consultou seus metadados com sucesso. `parentId` é exibido, sem criar associação automática. O hash usa arquivos ordenados por nome/tipo e sua serialização com fonte; não é baseline aprovada nem análise semântica. Código é apresentado como texto React e nunca executado.

SQLite guarda configuração pública, identidade, IDs, nomes, hash e horário/estado por serviço/conta. Não guarda valores, fórmulas ou fontes. As consultas não consomem vagas dos 30 monitores futuros. Sem credenciais, os testes de transporte são simulados; a homologação Google permanece pendente.

Fontes: [Sheets get](https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets/get), [batchGet](https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets.values/batchGet), [Drive files.get](https://developers.google.com/workspace/drive/api/reference/rest/v3/files/get), [Apps Script getContent](https://developers.google.com/apps-script/api/reference/rest/v1/projects/getContent).
