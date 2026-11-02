# Gate 1 — revisão independente dos adaptadores (tentativa 1/3)

Revisor: agente responsável pelo núcleo, sem autoria de `google.rs` ou `oauth.rs`. Revisão estática do código; execução de testes depende do log de testes principal. A revisão não usou credenciais nem fez chamadas autenticadas reais.

## Achado e correção

- **P2, limite de tempo do callback OAuth**: o timeout original era aplicado por leitura TCP. Um remetente local que enviasse fragmentos antes de cada timeout poderia manter `read_request` bloqueado para além do prazo de consentimento. Reportado ao autor e corrigido: `PendingAuthorization::wait` passa o menor prazo entre o deadline OAuth e dois segundos por request; `read_request` recalcula o tempo restante antes de cada leitura. Teste de loopback parcial adicionado. A correção foi relida e preserva validação de state, Host, caminho e uso único do verificador.

## Verificações estáticas

- URLs de destino Google fixas, HTTPS, sem redirecionamentos; validação antes do envio de bearer.
- IDs limitados a caracteres seguros; nenhuma operação de escrita em Sheets, Drive ou Apps Script.
- Coleta descarta resultados parciais se permissões, grid ou script autorizado falharem; paginação tem limite e detecção de ciclos.
- PKCE S256, state aleatório, bind loopback efêmero, consentimento no navegador do sistema e validação de parâmetros duplicados.
- Access/refresh token ficam em Rust e no Windows Credential Manager; mensagens não incluem corpo bruto de erro Google.
- Confirmação documental: `drive.metadata.readonly` é escopo admitido por [`permissions.list`](https://developers.google.com/workspace/drive/api/reference/rest/v3/permissions/list).

## Limites e condições para o runtime

- Capturar modo, conta e caminho do banco no início da coleta. Não recalcular o banco com configurações alteradas enquanto uma coleta está em andamento.
- Não restaurar `client_secret` de JSON comum: `OAuthConfig` omite o segredo na serialização. É necessário cofre seguro ou nova entrada pelo usuário quando aplicável.
- Uma coleta de várias APIs não é uma transação global do Google; os recursos podem mudar entre chamadas. A transação SQLite só torna atômica a persistência local.
- Token exchange, refresh/revogação reais, comportamento do Windows Credential Manager e acesso a contas/projetos reais permanecem **não verificados** sem credenciais e consentimento.

Nenhum outro bloqueio material foi identificado nesta revisão estática. Isso não equivale à conclusão da integração Google real.
