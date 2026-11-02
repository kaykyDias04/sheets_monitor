# Configuração OAuth — Sprint 02

Esta configuração depende de um projeto e uma conta autorizados. Não cole senhas, authorization codes, access tokens ou refresh tokens no chat ou no código. O consentimento e as verificações de segurança são feitos pelo usuário no navegador do sistema.

## Situação encontrada

A auditoria local não encontrou Google Cloud CLI, configuração autenticada do SDK nem variáveis de cliente/projeto da aplicação. Isso não afirma que o usuário não possui um projeto: apenas não existe acesso disponível ao agente por esses mecanismos. Evidência sanitizada em `docs/evidence/sprint02/access-audit.json`.

Nenhum projeto foi criado, API habilitada, consentimento concedido ou faturamento alterado. Não usar credenciais de um projeto diferente apenas porque estão disponíveis na máquina.

## Preparação no projeto correto

1. Selecionar o projeto existente da aplicação em [APIs e serviços](https://console.cloud.google.com/apis/dashboard). Só criar outro se o responsável confirmar que não existe projeto adequado.
2. Habilitar Google Sheets API, Google Drive API e Google Apps Script API. Isso não inclui configurar faturamento.
3. Configurar a audiência/tela de consentimento na [Google Auth Platform](https://console.cloud.google.com/auth/overview). Quando o projeto estiver em teste, incluir a conta de teste conforme as regras do Google/Workspace.
4. Criar um cliente OAuth do tipo **Desktop / Aplicativo para computador**. Cliente Web, service account e API key não substituem esse fluxo. [Documentação desktop](https://developers.google.com/identity/protocols/oauth2/native-app).
5. Configurar esse cliente na área Google do aplicativo. O client ID identifica o aplicativo; se a credencial Desktop emitida exigir client secret, inseri-lo somente no campo protegido da aplicação. Não usar arquivos versionados, `.env` do repositório ou mensagens para compartilhar o secret.

O armazenamento protegido e o fluxo são implementados no backend. A configuração por si só não é login. A tela deve informar falha ao salvar no cofre, sem fallback em texto simples.

## Permissões

Solicitar `openid`, `email`, `profile`, `spreadsheets.readonly` e `script.projects.readonly`. Drive é opcional e usa `drive.metadata.readonly` quando o usuário habilita a leitura de metadados; sua abrangência restrita sobre metadados deve ser considerada antes do consentimento. Não solicitar `drive`, `drive.readonly` ou permissão de escrita. A justificativa está no [ADR 0007](adr/0007-google-scopes-and-desktop-auth.md).

O OAuth desktop não suporta autorização incremental. Para mudar a opção Drive, realizar nova autorização completa no navegador. Respeitar escopos recusados e políticas do administrador; não contornar bloqueios de segurança. [Orientação oficial de escopos](https://developers.google.com/workspace/drive/api/guides/api-specific-auth).

## Acesso adicional a Apps Script

Além de habilitar a API no projeto Cloud e consentir o escopo, a conta pode precisar habilitar acesso da Apps Script API em [configurações do Apps Script](https://script.google.com/home/usersettings). Essa configuração não equivale ao consentimento do aplicativo e afeta acesso por aplicações autorizadas; o titular deve revisá-la. [Explicação oficial](https://developers.google.com/apps-script/api/how-tos/enable).

Usar script ID explicitamente autorizado; não assumir que ele é o spreadsheet ID nem prometer descoberta universal de scripts vinculados. O monitor somente lerá metadados e fontes; não executará ou modificará projetos.

## Checkpoint humano agrupado

Quando o build estiver pronto: configurar o cliente Desktop localmente, conectar a conta pelo navegador, consentir os escopos desejados e informar uma planilha e um projeto Apps Script de teste autorizados. Links/IDs podem ser informados ao aplicativo; conteúdos privados não devem entrar nas evidências versionadas.

O roteiro real deve comprovar identificação, leitura Sheets de abas/valores/fórmulas, metadados Drive quando autorizado, conteúdo Apps Script, reinício e renovação da sessão. Registrar apenas resultados sanitizados. Sem esses passos, a homologação real continua pendente, mesmo que todos os mocks passem.
