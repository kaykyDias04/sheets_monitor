# ADR 0007 — autorização Google desktop e menor privilégio

Estado: aceito para implementação da Sprint 02; homologação real separada e pendente até evidência.

O usuário autorizou OAuth desktop, leitura das três APIs e identificação da conta, sem edição/exclusão ou servidor próprio. O código arquivado não atende sozinho aos critérios atuais e não será reativado sem revisão.

A autorização usará PKCE S256, navegador do sistema, state de alta entropia e callback em `127.0.0.1` com porta dinâmica. O backend faz a troca e gerencia os tokens; o frontend recebe somente estado, conta e capacidades. Identidade vem do UserInfo autenticado; JWT não validado não é fonte de identidade. [Fluxo desktop](https://developers.google.com/identity/protocols/oauth2/native-app), [UserInfo](https://developers.google.com/identity/openid-connect/reference).

Escopos base: `openid`, `email`, `profile`, `spreadsheets.readonly` e `script.projects.readonly`. O acesso Drive é opt-in, com `drive.metadata.readonly`. Metadados são suficientes para SM-010; não precisamos do conteúdo de arquivos. Esse escopo é restrito e deve ser explicado ao titular antes da autorização. [Política de escopos Drive](https://developers.google.com/workspace/drive/api/guides/api-specific-auth).

`drive.file` foi avaliado: permite alterações e depende de o arquivo ser disponibilizado ao aplicativo; colar uma URL arbitrária não equivale a selecioná-lo pelo Picker. Não adotaremos permissões de escrita para resolver uma leitura de metadados. `drive.readonly` também não é necessário porque inclui conteúdo. [Semântica dos metadados](https://developers.google.com/workspace/drive/api/guides/file-metadata).

O Google não oferece autorização incremental para apps instalados. Mudar a opção Drive exige nova autorização completa e explícita, não uma ampliação silenciosa. Permissões recusadas devem aparecer por serviço e não ser confundidas com API aprovada. [Limite documentado para apps instalados](https://developers.google.com/identity/protocols/oauth2/native-app).

Tokens e client secret opcional serão protegidos pelo cofre Windows, sem fallback para JSON, SQLite ou localStorage. Logout remove sessão local; revogação remota é ação separada e explícita. Falha transitória não deve apagar uma credencial válida. Conta, cliente e geração da sessão delimitam operações concorrentes.

Consequências: testes com transporte/cofre simulados cobrem a lógica, e uma entrada sintética isolada pode comprovar o cofre local. Isso não comprova login, refresh ou respostas Google reais. Sem cliente/projeto/consentimento disponíveis, os critérios reais de SM-007 a SM-011 permanecem pendentes; nenhum faturamento ou projeto Cloud será criado por suposição.

## Atualização após homologação

O estado de uma conta em execução não representa o aceite da sprint. Removido o campo fixo real_validation do DTO e a mensagem fixa de pendência da interface; os resultados de engenharia ficam em docs/evidence/sprint02/google-live.json. A interface apresenta autorização e diagnósticos das consultas atuais. Nenhum estado global de “aprovado” é aplicado a contas futuras por causa desse teste.
