# Sheets Monitor — Sprint 02: Autenticação Google e integração com APIs

ETAPA 02 DE 06 · INSTRUÇÕES PARA CODEX · WINDOWS 11

A Sprint 02 transforma a estrutura criada na Sprint 01 em um aplicativo capaz de autenticar uma conta Google e consultar informações reais de planilhas, arquivos e projetos Apps Script.

O desenvolvimento continuará 100% conduzido pelo Codex, com autonomia para configurar o ambiente, escrever código, executar testes e corrigir problemas. A participação humana será limitada às autorizações de conta e segurança que não puderem ser delegadas.

Duração estimada

## 4–7 dias úteis

Tarefas

## SM-007 a SM-011

APIs integradas

## 3 serviços Google

Resultado

## Conexão real

Pré-requisito: Sprint 01 concluída, com projeto Tauri executável, banco SQLite e navegação funcional.

## 1. Objetivos e critérios de conclusão

| ID     | Requisito              | Critério de aceite                              |
| ------ | ---------------------- | ----------------------------------------------- |
| SM-007 | Login Google OAuth 2.0 | Autenticar e identificar a conta                |
| SM-008 | Persistência segura    | Recuperar sessão e renovar tokens               |
| SM-009 | Google Sheets API      | Ler metadados, abas, valores e fórmulas         |
| SM-010 | Google Drive API       | Consultar metadados e alterações de arquivos    |
| SM-011 | Google Apps Script API | Ler metadados e código dos projetos autorizados |

Ao concluir esta sprint, será possível informar o link de uma planilha real e consultar seus dados, sem ainda criar o sistema definitivo de monitoramento ou classificação Online/Offline.

## 2. Decisões técnicas para o Codex

### Autenticação Google

O aplicativo utilizará OAuth 2.0 para aplicativos desktop, com PKCE, navegador do sistema e redirecionamento local por loopback (`127.0.0.1`).

Essa abordagem é compatível com aplicativos Windows e evita capturar a senha do usuário dentro do aplicativo.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers



O Codex deverá implementar renovação de tokens, logout, revogação de acesso quando aplicável e armazenamento seguro das credenciais.

### Permissões mínimas

| Serviço                | Escopo inicial                                                                                   |
| ---------------------- | ------------------------------------------------------------------------------------------------ |
| Google Sheets          | `spreadsheets.readonly`                                                                          |
| Google Apps Script     | `script.projects.readonly`                                                                       |
| Google Drive           | Escopo mínimo suficiente para os metadados necessários, priorizando `drive.file` quando adequado |
| Identificação da conta | `openid`, `email`, `profile`                                                                     |

O escopo `drive.readonly` é mais abrangente e restrito; não deverá ser solicitado automaticamente sem justificativa técnica. A política de permissões deverá ser documentada.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers



### Requisito especial de Apps Script

A Apps Script API poderá exigir uma autorização adicional de acesso aos projetos no painel do Apps Script. Essa permissão é distinta do consentimento OAuth do aplicativo.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers



Nesta sprint, o Codex deve implementar leitura de código via `projects.getContent`, mas não edição ou execução de scripts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers



## 3. Sequência de implementação

02.1

Auditar a Sprint 01

Inspecionar estrutura, dependências, testes e arquitetura existente.

02.2

Configurar Google Cloud

Verificar projeto, ativar APIs e preparar credenciais OAuth por mecanismos oficiais.

02.3

Implementar autenticação

Criar fluxo PKCE, callback local, gerenciamento de sessão e armazenamento seguro.

02.4

Integrar Google Sheets

Implementar leitura de metadados, abas, intervalos, fórmulas e valores.

02.5

Integrar Google Drive

Consultar metadados dos arquivos e tratar erros de permissões.

02.6

Integrar Apps Script

Consultar projetos por ID e recuperar código autorizado.

02.7

Criar interface de conexão

Adicionar login, conta conectada, diagnóstico de APIs e teste por URL.

02.8

Executar testes

Validar segurança, integração real, falhas de rede e recuperação de sessão.

# 4. Prompt 02 — Implementação completa no Codex

Copie o prompt abaixo e envie ao Codex após a conclusão da Sprint 01.

# PROMPT 02 — Autenticação Google e integração com Google Workspace APIs

Você é o agente de engenharia responsável pela Sprint 02 do Sheets Monitor.

Execute esta sprint de maneira autônoma, respeitando integralmente o contrato mestre estabelecido no Prompt 00.

## 1. CONTEXTO

O Sheets Monitor é um aplicativo desktop Windows 11, desenvolvido com:

- Tauri 2
- React + TypeScript
- Rust
- SQLite
- Tailwind CSS + shadcn/ui

Seu objetivo final é monitorar até 30 planilhas Google Sheets, incluindo estruturas, fórmulas, alterações de conteúdo e dependências de Apps Script.

A Sprint 01 criou a fundação do aplicativo.

Nesta Sprint 02, implemente a autenticação Google e as integrações reais com Google Sheets API, Google Drive API e Google Apps Script API.

## 2. AUTONOMIA OPERACIONAL

Você está autorizado a:

1. Inspecionar integralmente o repositório.
2. Instalar dependências necessárias e permitidas.
3. Criar e modificar arquivos.
4. Implementar módulos Rust e TypeScript.
5. Executar comandos, builds e testes.
6. Automatizar configurações locais e do Google Cloud quando houver acesso previamente autorizado.
7. Consultar documentações oficiais atualizadas.
8. Corrigir erros autonomamente.
9. Atualizar documentação técnica.
10. Criar commits em uma branch de trabalho.

Não solicite confirmação para decisões técnicas rotineiras e reversíveis dentro do escopo da Sprint 02.

Não realize operações externas sensíveis sem a autorização necessária.

Nunca solicite que o usuário envie senhas, tokens de acesso, refresh tokens ou outros segredos para o chat.

Não realize alterações em produção fora do escopo autorizado.

## 3. AUDITORIA PRÉVIA

Antes de implementar:

1. Leia AGENTS.md e a documentação existente.
2. Inspecione as dependências instaladas.
3. Verifique a estrutura de frontend e backend.
4. Verifique as migrações SQLite.
5. Execute os testes da Sprint 01.
6. Identifique incompatibilidades.
7. Corrija problemas que bloqueiem a integração.

Não recrie o projeto se já houver uma estrutura funcional.

Preserve as decisões arquiteturais aprovadas.

## 4. CONFIGURAÇÃO GOOGLE CLOUD

Verifique se existe um projeto Google Cloud autorizado para esta aplicação.

Se houver credenciais e permissões apropriadas:

1. Identifique o projeto.
2. Verifique as APIs habilitadas.
3. Habilite Google Sheets API.
4. Habilite Google Drive API.
5. Habilite Google Apps Script API.
6. Verifique a configuração de consentimento OAuth.
7. Prepare um cliente OAuth do tipo Desktop.
8. Configure os escopos mínimos necessários.

Se alguma operação exigir autenticação humana, consentimento, privilégio adicional ou acesso indisponível, apresente apenas a ação indispensável.

Não crie projetos duplicados desnecessariamente.

Não configure faturamento sem autorização explícita.

Não coloque segredos em arquivos versionados.

## 5. AUTENTICAÇÃO OAUTH 2.0

Implemente um fluxo OAuth 2.0 nativo para aplicativo desktop.

Requisitos:

- Authorization Code Flow com PKCE S256.
- Navegador externo do sistema.
- Callback por loopback local.
- Porta local dinâmica.
- Validação do parâmetro state.
- Validação dos parâmetros de resposta.
- Troca segura do authorization code.
- Tratamento de expiração de sessão.
- Renovação automática do access token quando autorizada.
- Armazenamento protegido de refresh tokens.
- Logout.
- Revogação de autorização, quando solicitada.
- Tratamento de consentimento negado.
- Tratamento de erros de rede.
- Proteção contra exposição de tokens em logs.

Use bibliotecas mantidas e mecanismos compatíveis com Windows 11.

Implemente a lógica sensível preferencialmente no backend Rust.

Não exponha refresh tokens ao frontend React.

## 6. ESCOPO DE AUTORIZAÇÃO

Solicite somente as permissões necessárias para leitura.

Escopos iniciais:

- openid
- email
- profile
- [https://www.googleapis.com/auth/spreadsheets.readonly](https://www.googleapis.com/auth/spreadsheets.readonly)
- [https://www.googleapis.com/auth/script.projects.readonly](https://www.googleapis.com/auth/script.projects.readonly)

Para o Google Drive, avalie primeiro a viabilidade de drive.file para os recursos efetivamente disponibilizados ao aplicativo.

Caso seja tecnicamente necessário usar um escopo mais amplo, documente a justificativa, impacto de privacidade e possíveis requisitos de verificação OAuth.

Não solicite permissões de edição ou exclusão de planilhas.

Implemente tratamento de permissões ausentes e autorização incremental quando apropriado.

## 7. INTEGRAÇÃO COM GOOGLE SHEETS API

Crie um módulo Rust dedicado ao Google Sheets.

Funcionalidades:

1. Extrair spreadsheetId de URLs válidas.
2. Validar IDs e links recebidos.
3. Consultar metadados da planilha.
4. Listar todas as abas.
5. Recuperar sheetId, título, posição e propriedades relevantes.
6. Consultar intervalos específicos.
7. Recuperar valores calculados.
8. Recuperar fórmulas com FORMULA.
9. Ler múltiplos intervalos por batchGet.
10. Tratar respostas vazias e intervalos inválidos.
11. Tratar erros 400, 401, 403, 404, 429 e 5xx.
12. Implementar timeout e retentativas seguras.

Crie modelos tipados para as respostas da API.

Não implemente ainda a classificação estrutural Online/Offline.

Não crie baselines definitivos nesta sprint.

## 8. INTEGRAÇÃO COM GOOGLE DRIVE API

Crie um módulo dedicado à leitura de metadados.

Implementar:

- Consulta de metadados por fileId.
- Nome do arquivo.
- Tipo MIME.
- Data de modificação, quando disponível.
- Identificadores relevantes.
- Verificação de acesso.
- Tratamento de arquivos removidos ou inacessíveis.

Não presuma que os metadados do Drive fornecem detalhes de todas as alterações em células.

Não trate modifiedTime como prova suficiente de alteração estrutural.

## 9. INTEGRAÇÃO COM GOOGLE APPS SCRIPT API

Crie um módulo dedicado a projetos Apps Script.

Implementar:

1. Cadastro ou associação de scriptId.
2. Consulta de metadados via projects.get.
3. Leitura do código via projects.getContent.
4. Recuperação de arquivos .gs, .html e manifesto, conforme retornados.
5. Identificação de nomes e tipos dos arquivos.
6. Tratamento de projetos sem permissão.
7. Tratamento de erros e APIs desabilitadas.
8. Representação tipada do conteúdo retornado.
9. Geração de hash local do código para futuras comparações.

A associação entre uma planilha e um projeto Apps Script deve ser explícita ou comprovada.

Não assuma que o spreadsheetId é igual ao scriptId.

Não prometa descoberta automática universal dos projetos vinculados.

Não execute nem modifique scripts.

A análise sintática e estrutural aprofundada será desenvolvida na Sprint 03.

## 10. INTERFACE DO APLICATIVO

Implemente uma área de integração Google com:

- Botão "Conectar conta Google".
- Identificação da conta autenticada.
- Estado de autenticação.
- Botão "Desconectar".
- Indicador de conexão com os serviços.
- Campo para inserir URL de uma planilha.
- Botão "Testar acesso".
- Exibição de nome e abas da planilha.
- Indicador de acesso à Sheets API.
- Indicador de acesso à Drive API.
- Campo opcional para scriptId.
- Botão "Testar Apps Script".
- Diagnóstico de permissões insuficientes.

Os componentes devem apresentar estados de carregamento, sucesso e erro.

Nenhum dado simulado poderá ser apresentado como resultado de uma chamada real.

## 11. PERSISTÊNCIA LOCAL

Aproveite o SQLite existente.

Implemente persistência de:

- Identificador da conta Google.
- Informações não sensíveis da conta.
- IDs das planilhas consultadas.
- IDs de scripts associados.
- Configuração de integração.
- Estado e horário da última consulta.

Não armazene senhas, tokens OAuth ou segredos em texto simples no banco.

Utilize armazenamento seguro compatível com o Windows para credenciais sensíveis.

## 12. TESTES AUTOMATIZADOS

Crie testes unitários e de integração para:

- Extração de spreadsheetId.
- URLs inválidas.
- Geração de PKCE.
- Validação de state.
- Processamento do callback.
- Troca e renovação de tokens com respostas simuladas.
- Falhas de autenticação.
- Respostas válidas e inválidas do Sheets.
- Respostas válidas e inválidas do Drive.
- Respostas válidas e inválidas do Apps Script.
- Erros 401, 403, 404, 429 e 5xx.
- Timeouts.
- Falha de internet.
- Ausência de permissão.
- Persistência de dados não sensíveis.
- Ausência de vazamento de credenciais.

Utilize mocks e fixtures para testes que não exigem acesso Google real.

Não considere os testes simulados como validação de integração real.

## 13. TESTES DE INTEGRAÇÃO REAL

Quando existir uma conta Google autorizada:

1. Autentique a conta.
2. Recupere sua identificação.
3. Consulte uma planilha de teste.
4. Liste suas abas.
5. Leia valores de um intervalo.
6. Leia fórmulas de um intervalo.
7. Consulte metadados do Drive, se autorizado.
8. Consulte um projeto Apps Script de teste, se autorizado.
9. Reinicie o aplicativo.
10. Verifique a persistência segura da sessão.
11. Verifique a renovação do token.

Registre resultados e erros sem expor conteúdo sensível.

Se as credenciais não estiverem disponíveis, execute os testes automatizados possíveis e classifique a integração real como pendente.

Não declare a integração real aprovada sem testá-la.

## 14. DOCUMENTAÇÃO

Atualize:

- README.md
- docs/oauth-setup.md
- docs/google-apis.md
- docs/security.md
- docs/api-contracts.md
- docs/testing.md

Documente:

- Arquitetura da autenticação.
- Permissões solicitadas.
- Fluxo OAuth.
- Configuração Google Cloud.
- Tratamento de tokens.
- Operações das APIs.
- Limitações de Apps Script.
- Procedimentos de teste.
- Pendências humanas.

## 15. CRITÉRIOS DE ACEITE

A Sprint 02 será considerada concluída quando:

1. O aplicativo permitir login Google real.
2. A conta autenticada for identificada corretamente.
3. A sessão puder ser recuperada com segurança.
4. A Sheets API retornar dados reais de planilha autorizada.
5. A Drive API retornar metadados quando autorizada.
6. A Apps Script API retornar código de projeto autorizado.
7. Os erros de acesso forem tratados corretamente.
8. Os tokens não forem expostos ou armazenados em texto simples.
9. Os testes automatizados passarem.
10. O aplicativo compilar no Windows 11.
11. A documentação estiver atualizada.

Se uma autorização externa impedir determinada integração, marque o respectivo critério como pendente, sem declarar sucesso.

## 16. RELATÓRIO FINAL

Ao concluir, apresente:

- Funcionalidades implementadas.
- Arquivos criados ou modificados.
- APIs integradas.
- Escopos utilizados.
- Testes executados e seus resultados.
- Evidências das integrações reais.
- Pendências humanas.
- Problemas de segurança identificados.
- Recomendações para a Sprint 03.

## 17. RESTRIÇÃO DE ESCOPO

NÃO implemente a Sprint 03.

Não desenvolva ainda:

- Motor definitivo de integridade.
- Comparação completa de baselines.
- Classificação automática Online/Offline.
- Análise semântica avançada de Apps Script.
- Agendamento contínuo.
- Notificações definitivas.

Esses recursos serão implementados nas próximas sprints.

Ao concluir a Sprint 02, encerre a execução e aguarde o Prompt 03.

# 5. Intervenções humanas indispensáveis

O Codex deverá tentar realizar todas as configurações possíveis com ferramentas oficiais e credenciais já autorizadas. Ainda assim, algumas operações poderão exigir sua participação.

Checklist de autorizações

0/5

Autenticar a conta Google

Concluir login e eventuais verificações de segurança no navegador.

Autorizar o aplicativo

Aprovar o consentimento OAuth e os escopos solicitados.

Permitir acesso aos projetos Apps Script

Habilitar a configuração pertinente, se exigida pelo Google.

Disponibilizar planilha e script de teste

Criar ou autorizar o uso de arquivos fictícios, sem dados de produção.

Autorizar ações administrativas

Aprovar operações do Google Cloud ou Windows que exigirem intervenção.

Se você precisar realizar manualmente alguma configuração, utilize estes painéis oficiais:

- [Google Cloud — APIs e Serviços](https://console.cloud.google.com/apis/dashboard?utm_source=chatgpt.com)
- [Google Auth Platform — Configuração OAuth](https://console.cloud.google.com/auth/overview?utm_source=chatgpt.com)
- [Google Apps Script — Configurações do usuário](https://script.google.com/home/usersettings?utm_source=chatgpt.com)

Para minimizar ainda mais a intervenção humana, o Codex deverá reunir todas as autorizações pendentes em um único checkpoint, sempre que possível, em vez de interromper o desenvolvimento repetidamente.

## 6. Gate de encerramento

A Sprint 02 não deverá ser aprovada apenas porque o aplicativo exibe o botão de login. É indispensável demonstrar que a autenticação funciona, que a sessão é armazenada com segurança e que as três APIs retornam dados reais de recursos autorizados.

Se alguma integração depender de uma permissão não concedida, o Codex deverá registrar a pendência e deixar o módulo implementado e testado por mocks, distinguindo-o de uma integração real homologada.

Próxima etapa: Sprint 03 — Motor de integridade estrutural, comparação de baselines, detecção de fórmulas quebradas e análise de dependências de Apps Script. Essa será a etapa central para determinar quando uma planilha deve ficar Online ou Offline.