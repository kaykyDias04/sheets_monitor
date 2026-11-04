# OAuth para distribuição — mantenedor

O cliente Desktop pertence ao projeto Google autorizado da aplicação. Usuário final entra com Google; não deve criar projeto Cloud nem fornecer credenciais de desenvolvimento. Ver [configuração detalhada](oauth-setup.md).

Provisione `SHEETS_MONITOR_OAUTH_CLIENT_ID` e, se exigido pelo cliente Desktop, `SHEETS_MONITOR_OAUTH_DESKTOP_SECRET` somente no ambiente privado de build autorizado. Não ler o cofre do usuário para fabricar provisionamento. O relatório registra apenas presença, sem valores. Cliente instalado é público: segredo Desktop incorporado não é segredo de servidor; PKCE continua obrigatório. Nunca incorporar refresh/access tokens, conta de teste ou banco real.

A configuração inicial não substitui uma configuração protegida já existente. Uma sessão antiga funcionando no computador do mantenedor não comprova primeiro acesso do artefato em outro perfil. Build sem provisão deve permanecer explicitamente não pronto para novos usuários.

Escopos: openid/email/profile, spreadsheets.readonly, script.projects.readonly; drive.metadata.readonly apenas com opção de descoberta. Não usar escopos de escrita, drive.file com escrita ou autorização incremental incompatível com app instalado. Consentimento ocorre no navegador do sistema. Políticas Workspace, audiência, modo de teste e verificação dos escopos restritos/privacidade precisam de revisão no projeto correto antes de publicação externa. Não criar projetos duplicados, configurar faturamento ou contornar bloqueios.

A homologação deve comprovar login no candidato, leitura das três APIs, reinício e refresh no ambiente autorizado, com relatório sanitizado. Não revogar concessão nem alterar preferências para simular falha sem ação explícita. Recursos e referências para carga devem ser preparados/aprovados pelo usuário no aplicativo; não enviar IDs/conteúdos/tokens ao relatório público.

Na execução original não foram fornecidos ambiente separado nem30 recursos aprovados. Não houve leitura do cofre para provisionar o candidato. A compilação da integração atual será registrada em `docs/evidence/release-integration/artifact-manifest.json`. A autorização de publicação atual e seus bloqueios estão registrados na atualização abaixo.


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.


### Atualização — audiência OAuth interna

O usuário informou que o console Google dispensa verificação porque o projeto está configurado com tipo de usuário Interno. Nesse cenário, não exigir aprovação pública como pré-condição: o OAuth limita-se aos membros da organização Google Cloud/Workspace do projeto e às políticas administrativas correspondentes. Fonte: https://support.google.com/cloud/answer/15549945#user-type-internal . A informação é relato do usuário; o nome/ID do projeto e o JSON Desktop ainda não foram fornecidos. O bloqueio atual é obter a configuração Desktop autorizada e comprovar login do candidato com conta interna elegível, além das homologações já pendentes. Não alterar a audiência para Externo nem ampliar escopos automaticamente. A futura release deve informar que o login é restrito à organização.

### Configuração fornecida após a integração

O usuário identificou o projeto interno autorizado e forneceu o JSON Desktop por caminho local. A configuração foi validada e incorporada ao novo candidato; relatório em `docs/evidence/release-oauth/report.md`. Não publicar valores, caminho privado ou cópia do JSON. O rascunho de release contém apenas instalador,checksums e manifesto sanitizado. A pendência atual é homologar o novo candidato com conta interna elegível e concluir os demais critérios, sem exigir verificação pública indevida.

## Publicação OAuth interna

A configuração já foi incorporada e o usuário confirmou login e leitura no novo instalador. Pré-release1.0.0 publicada,com limites de audiência interna e homologação integral explicitados. Não é necessária verificação pública para esse público; contas externas à organização permanecem fora da audiência. O JSON privado não é asset de distribuição; configuração de cliente Desktop é incorporada ao binário conforme o protocolo instalado. Evidências sanitizadas em docs/evidence/release-oauth.


## Distribuição portátil entre máquinas Windows 11 (v1.0.3+)

Para garantir que o instalador compilado autentique em qualquer computador Windows 11 sem configuração manual por máquina (`.env`, variáveis de ambiente ou credenciais prévias no cofre do desenvolvedor):

1. **Configuração pública versionada (`src-tauri/oauth-client.json`)**: armazena o `client_id` público do cliente OAuth do tipo **Desktop App** (`installed`), acompanhando automaticamente o código-fonte e o build Tauri sem depender de caminhos absolutos da máquina principal.
2. **Resolução e validação em tempo de compilação (`src-tauri/build.rs`)**:
   - Aceita variáveis de processo (`SHEETS_MONITOR_OAUTH_CLIENT_ID` / `SHEETS_MONITOR_OAUTH_DESKTOP_SECRET`), caminho explícito (`SHEETS_MONITOR_OAUTH_CLIENT_JSON`), arquivo local ignorado pelo Git (`.secrets/oauth-desktop.json` ou `.secrets/client_secret.json`) e o `src-tauri/oauth-client.json`.
   - Rejeita automaticamente JSONs do tipo `web` (aplicações confidenciais) e exige sufixo `.apps.googleusercontent.com`.
   - Quando `SHEETS_MONITOR_REQUIRE_OAUTH=1` está ativo (`scripts/build-release.ps1`), bloqueia o build caso o cliente OAuth Desktop não esteja provisionado, impedindo publicação acidental de instalador desconfigurado.
3. **Sessão individual por usuário do Windows (`WindowsVault`)**:
   - O cliente OAuth Desktop distribuído é compartilhado entre instalações como cliente público com **Authorization Code + PKCE (`S256`)**, navegador externo do sistema e callback loopback em porta dinâmica (`http://127.0.0.1:<porta>/oauth/callback`).
   - Cada usuário autentica sua própria conta Google e tem seu `refresh_token` armazenado individualmente no Gerenciador de Credenciais do Windows (`SheetsMonitor.OAuth.v2`), com renovação automática de `access_token`, logout local (`Desconectar conta`) e revogação explícita (`Revogar acesso no Google`).
4. **Requisitos externos no Google Cloud Console**:
   - **APIs habilitadas no projeto**: Google Sheets API, Google Drive API e Google Apps Script API.
   - **Audiência / Tela de consentimento OAuth**:
     - **Interno (Internal)**: permite login apenas para contas pertencentes à mesma organização Google Workspace do projeto Cloud; contas `@gmail.com` externas recebem bloqueio `org_internal` pelo Google.
     - **Externo — Em teste (Testing)**: permite login apenas para os e-mails cadastrados explicitamente em *Test users* no Google Auth Platform (até 100 usuários).
     - **Externo — Em produção (In production)**: permite login com qualquer conta Google; como o aplicativo utiliza escopos restritos/sensíveis (`spreadsheets.readonly`, `script.projects.readonly` e opcionalmente `drive.metadata.readonly`), o uso por contas externas fora da lista de teste requer verificação OAuth do Google para evitar tela de bloqueio/aviso de app não verificado.
