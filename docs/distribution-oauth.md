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
