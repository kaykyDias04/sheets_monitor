> DOCUMENTO HISTÓRICO DO PROTÓTIPO. Este texto descreve código arquivado, não a fundação ativa nem uma integração real homologada. Consulte [requisitos atuais](requirements.md), [arquitetura](architecture.md) e [progresso](progress.md). A versão original permanece em `archive/prototype/` e no commit `ffcaa49`.

# Integração Google da V1

## Estado verificável

Adaptadores HTTPS implementados em Rust, com fixtures e testes unitários. Nenhuma
conta Google foi conectada durante a implementação. Nenhuma planilha, permissão ou
fonte Apps Script real foi alterada. Credenciais, consentimento, aprovação humana
da baseline e teste real continuam necessários para homologação da integração.

## Configurar uma conta

1. Criar ou selecionar um projeto Google Cloud controlado pelo usuário.
2. Habilitar Google Sheets API, Google Drive API e Apps Script API.
3. Configurar a tela de consentimento OAuth, incluindo usuários de teste quando
   o projeto estiver em teste. Políticas do Workspace podem bloquear escopos.
4. Criar credenciais OAuth do tipo **Aplicativo para computador**. Informar o
   client ID e, quando exigido pela credencial emitida pelo Google, client secret.
   Um aplicativo desktop não consegue garantir a confidencialidade desse secret;
   a proteção da autorização é PKCE e consentimento no navegador do sistema.
5. Acionar Conectar no aplicativo. O backend abre o navegador do sistema e espera
   até três minutos por um callback em `127.0.0.1`, porta efêmera. Não há servidor
   hospedado nem acesso de rede à interface de callback.
6. Autorizar somente os escopos de leitura solicitados. A coleta exige os três
   escopos; consentimento parcial não produz sessão utilizável.
7. Adicionar IDs de planilhas e, explicitamente, IDs dos projetos Apps Script que
   o usuário autoriza ler. A V1 não promete descobrir automaticamente scripts
   vinculados ou projetos que a conta não consegue acessar.
8. Revisar e aprovar a baseline coletada antes de iniciar comparações.

As credenciais nunca devem ser incluídas em commits, fixtures, relatórios ou logs.
O modelo `OAuthConfig` omite o client secret na serialização. Tokens de acesso e
refresh são persistidos exclusivamente no **Windows Credential Manager**, em
uma entrada associada ao hash do client ID. O client secret desktop opcional é
lido da variável de ambiente `SHEETS_MONITOR_CLIENT_SECRET` pelo backend; não é
persistido pelo aplicativo. Não há fallback de tokens para arquivo texto,
localStorage, frontend ou SQLite. Não há teste real do Credential Manager nesta
entrega até que uma conta seja conectada no aplicativo.

## Escopos e fronteiras

| API | Escopo solicitado | Consultas |
|---|---|---|
| Sheets | `spreadsheets.readonly` | `spreadsheets.get?includeGridData=true` |
| Drive | `drive.metadata.readonly` | `files.get` com metadados completos e `permissions.list` paginado |
| Apps Script | `script.projects.readonly` | `projects.getContent` para cada ID explicitamente autorizado |

Os caminhos de coleta usam apenas GET. O módulo não expõe mutações de planilhas,
Drive ou Apps Script. POST é usado exclusivamente nos endpoints OAuth de troca,
refresh e revogação. Revogar uma autorização é uma ação externa separada, que
deve ser solicitada explicitamente; desconectar normalmente apaga só a credencial
local. A autorização previamente concedida continua visível na conta Google.

Um refresh preserva o refresh token anterior quando o Google não emite outro.
Um **novo consentimento** nunca reutiliza o refresh token anterior, pois o usuário
pode ter selecionado outra conta. Configurações com outro client ID não acessam
as credenciais da configuração anterior.

Depois do consentimento o backend consulta `Drive about?fields=user(permissionId)`
e particiona o banco por SHA-256 de client ID e identidade estável da conta.
Reconectar a mesma conta mantém seu histórico; selecionar outra conta cria um
histórico separado, exigindo cadastro e baseline próprios. Antes de cada lote de
coleta a identidade é verificada novamente. Uma credencial trocada fora do fluxo
esperado produz erro, preservando o histórico anterior.

## Cobertura de coleta e diferenças

A coleta retorna Sheets com dados de grid, metadados Drive, todas as páginas de
permissões e fontes/manifest dos scripts explicitamente autorizados. O snapshot
é completo ou falha integralmente. Se uma API falha, a coleta parcial não deve
substituir a última captura nem a baseline. Remoção de acesso é falha operacional
visível, sem fabricar uma exclusão de todos os dados.

Permissions são ordenadas por ID e arquivos de um projeto por nome/tipo para
estabilizar a ordem de conjuntos retornados pela API. Nenhuma fonte é normalizada:
espaços, linhas e texto permanecem exatos. A ordem das abas e linhas permanece
significativa. Não há filtro por severidade para omitir diferenças capturadas.

O polling detecta diferenças entre capturas bem-sucedidas. Não há garantia de
observar edições intermediárias revertidas antes da próxima captura, alterações
em recursos não expostos pelas APIs, histórico completo de execução, gatilhos ou
projetos que não foram autorizados. `getContent` lê a versão HEAD, não executa
código e não oferece uma auditoria completa do comportamento de um script.

## Falhas e limites explícitos

- Timeouts de conexão de dez segundos; consulta de recurso de até 45 segundos.
- Resposta de recurso limitada a 64 MiB. Acima disso há erro explícito e a
  captura anterior é preservada; não há truncamento silencioso.
- Até quatro tentativas por consulta para 429, erros 5xx transitórios e razões
  Google `rateLimitExceeded`/`userRateLimitExceeded`. Backoff limitado; `Retry-After`
  numérico respeitado até cinco segundos. Não há loop infinito de quota.
- Erros 401 pedem reconexão. Erros 403/404 não retornam corpos da API ao frontend
  ou logs, pois esses corpos podem conter conteúdo sensível.
- Todos os requests rejeitam redirects. O bearer só pode ir a endpoints HTTPS
  fixos de Sheets, Drive e Apps Script. IDs rejeitam URLs, barras e query strings.
- Paginação interrompida, repetida ou acima de mil páginas falha explicitamente.
- Até trinta IDs de script por monitor, sem duplicatas. O limite de trinta
  planilhas é imposto pelo domínio/persistência, independentemente desta API.

## Evidências reproduzíveis

Os testes de `google.rs` usam `ReadTransport` falso, sem chamadas a Google:
paginação, ordenação, coleta de script autorizado, falha integral por script
indisponível, redigir erros, tentativas limitadas e bloqueio de exfiltração.
Os testes de `oauth.rs` verificam o vetor oficial PKCE, escopos readonly, bind
loopback, parâmetros de consentimento, validação de state/host/caminho/código,
timeout, preservação de refresh e isolamento do vault por client ID.

Resultados executados, versões e limitações do ambiente ficam no relatório de
evidências geral; a existência desses testes não significa que tenham passado
até que o comando correspondente tenha terminado com sucesso.

## Referências primárias

- [Google OAuth para aplicativos nativos](https://developers.google.com/identity/protocols/oauth2/native-app)
- [Suporte ao callback loopback para aplicativos desktop](https://developers.google.com/identity/protocols/oauth2/resources/loopback-migration)
- [Sheets spreadsheets.get](https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets/get)
- [Drive permissions.list](https://developers.google.com/workspace/drive/api/reference/rest/v3/permissions/list)
- [Drive seleção de campos](https://developers.google.com/workspace/drive/api/guides/fields-parameter)
- [Apps Script projects.getContent](https://developers.google.com/apps-script/api/reference/rest/v1/projects/getContent)
- [RFC 8252 — OAuth para aplicações nativas](https://www.rfc-editor.org/rfc/rfc8252)
- [RFC 7636 — PKCE](https://www.rfc-editor.org/rfc/rfc7636)
