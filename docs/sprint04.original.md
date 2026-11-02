# Sheets Monitor — Sprint 04: Interface e gerenciamento de planilhas

ETAPA 04 DE 06 · PROMPT PARA CODEX · WINDOWS 11

A Sprint 04 será responsável por transformar o motor de integridade desenvolvido na Sprint 03 em uma aplicação com interface funcional, permitindo visualizar planilhas, administrar regras de proteção, consultar modificações e analisar falhas.

Esta etapa incorporará também uma decisão importante tomada posteriormente ao planejamento original: o Sheets Monitor deverá descobrir automaticamente as planilhas disponíveis na conta Google conectada, sem exigir que o usuário cadastre manualmente os links.

O cadastro por URL permanecerá como alternativa.

Backlog

## SM-018 a SM-021

Estimativa

## 4–7 dias úteis

Capacidade

## 30 monitoradas

Prioridade

## Alta

Pré-requisito: motor de integridade e APIs Google implementados nas Sprints 02 e 03, com contratos de dados disponíveis.

## 1. Escopo funcional da Sprint 04

| ID      | Módulo       | Entrega                                           |
| ------- | ------------ | ------------------------------------------------- |
| SM-018  | Dashboard    | Visão geral e status das planilhas                |
| SM-019  | Cadastro     | Descoberta automática, cadastro por URL e remoção |
| SM-020  | Configuração | Seleção de abas, regras, fórmulas e dependências  |
| SM-021  | Histórico    | Consulta, filtros e diagnósticos de alterações    |
| SM-021A | Conta Google | Gerenciamento visual da conta conectada           |
| SM-021B | Baselines    | Criação, aprovação e comparação de versões        |

As tarefas SM-021A e SM-021B são adições necessárias para que o usuário consiga operar o aplicativo sem recorrer ao terminal ou executar comandos técnicos.

### Regra de uso diário

Ao abrir o aplicativo pela primeira vez, o usuário deverá encontrar o seguinte fluxo:

01

Entrar com Google

Autenticação no navegador do sistema

02

Descobrir planilhas

Listar os arquivos acessíveis mediante autorização

03

Selecionar arquivos

Escolher até 30 planilhas para monitorar

04

Configurar proteção

Escolher abas, colunas, fórmulas e scripts

05

Aprovar baseline

Confirmar a estrutura de referência

06

Acompanhar integridade

Consultar diagnóstico e histórico

A descoberta automática deverá ser implementada utilizando a Google Drive API, com `files.list`, filtro para planilhas Google e paginação. A API permite pesquisar arquivos por tipo MIME e selecionar os campos retornados.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers+1



Atenção: o escopo OAuth `drive.file`, preferido inicialmente por conceder acesso limitado, não é suficiente para garantir uma listagem geral de todas as planilhas acessíveis à conta. O Codex deverá avaliar e implementar o escopo de leitura apropriado para descoberta ampla, documentando o impacto sobre a verificação e publicação do aplicativo Google. Não deverá apresentar uma lista parcial como se fosse completa.

## 2. Estrutura das telas

O aplicativo terá seis áreas principais.

Sheets Monitor

Visão geralPlanilhasAtividadesRegrasConfigurações

# 24

Cadastradas

# 21

Íntegras

# 3

Com falhas

Últimas verificações

Orçamento 2026

Online

Empenhos

Offline

BD Auxiliar

Online

Protótipo ilustrativo e navegável. Não está conectado a planilhas reais.

Além dessas áreas, haverá uma tela de detalhes por planilha, acessível ao clicar em qualquer arquivo cadastrado.

Essa tela deverá exibir a lista de abas, a versão do baseline, diagnósticos, regras de proteção, scripts associados, última verificação e histórico filtrado.

# 3. Prompt 04 — Implementação completa no Codex

O texto abaixo é a instrução operacional desta sprint. Ele pressupõe que o Codex já tenha acesso ao repositório e às implementações anteriores.

# PROMPT 04 — Interface, descoberta automática e gerenciamento de planilhas

Você é o agente principal de engenharia do projeto Sheets Monitor, responsável pela Sprint 04.

Execute integralmente esta etapa, respeitando o contrato de autonomia estabelecido no Prompt 00 e preservando as funcionalidades implementadas nas Sprints 01, 02 e 03.

## 1. CONTEXTO

O Sheets Monitor é um aplicativo desktop para Windows 11, desenvolvido com:

- Tauri 2
- React e TypeScript
- Rust
- SQLite
- Tailwind CSS
- shadcn/ui

O aplicativo deverá monitorar até 30 planilhas Google Sheets, incluindo integridade estrutural, fórmulas, alterações de conteúdo e dependências de Apps Script.

As sprints anteriores implementaram a fundação, autenticação Google, integrações com APIs e motor de integridade.

Esta sprint deverá construir a interface operacional definitiva e o gerenciamento de planilhas.

## 2. AUTONOMIA

Execute autonomamente todas as tarefas técnicas possíveis:

- Inspeção do repositório.
- Instalação de dependências.
- Criação e modificação de código.
- Atualização do banco de dados.
- Implementação de componentes.
- Integração de frontend e backend.
- Execução de testes.
- Correção de erros.
- Documentação.
- Commits na branch de trabalho.

Não solicite confirmação para decisões técnicas rotineiras e reversíveis.

Não modifique dados reais de planilhas.

Não solicite senhas ou tokens privados ao usuário.

Interrompa apenas operações que efetivamente dependam de autorização humana.

Não avance automaticamente para a Sprint 05.

## 3. AUDITORIA PRÉVIA

Antes de implementar:

1. Leia AGENTS.md.
2. Revise a documentação do projeto.
3. Execute os testes existentes.
4. Identifique os contratos Tauri implementados.
5. Identifique as operações disponíveis no motor de integridade.
6. Inspecione a integração OAuth e os escopos autorizados.
7. Revise o esquema SQLite.
8. Identifique componentes visuais reutilizáveis.

Preserve a arquitetura existente.

Corrija apenas problemas que bloqueiem os requisitos da sprint.

## 4. NOVO REQUISITO OBRIGATÓRIO — DESCOBERTA AUTOMÁTICA

O usuário final deverá conseguir:

1. Abrir o aplicativo.
2. Entrar com sua conta Google.
3. Visualizar as planilhas às quais o aplicativo recebeu autorização para acessar.
4. Selecionar quais deseja monitorar.
5. Configurar abas e regras.
6. Aprovar a estrutura inicial.

O usuário não deverá precisar informar:

- Google Cloud Project ID.
- OAuth Client ID.
- OAuth Client Secret.
- IDs de projetos Google Cloud.
- Chaves de API.
- IDs de planilhas, exceto quando optar por cadastro manual via link.
- IDs de scripts, exceto quando não houver meio autorizado de associá-los automaticamente.

O projeto Google Cloud e a configuração OAuth pertencem à aplicação e deverão ser preparados durante o desenvolvimento e empacotamento.

O usuário final somente precisará conceder as permissões exigidas pelo Google.

## 5. GOOGLE DRIVE — DESCOBERTA DE PLANILHAS

Implemente a descoberta de planilhas utilizando Google Drive API v3.

Utilize files.list com:

- Filtro MIME de planilhas Google.
- Exclusão de arquivos na lixeira.
- Paginação com nextPageToken.
- Seleção explícita de campos retornados.
- Consulta de identificadores, nomes e metadados pertinentes.
- Suporte a arquivos compartilhados conforme as permissões disponíveis.
- Suporte a unidades compartilhadas quando autorizado e aplicável.
- Tratamento de resultados incompletos.
- Tratamento de erros de autorização.

O aplicativo deverá conseguir listar mais de 30 arquivos disponíveis, pois o limite de 30 se aplica apenas aos arquivos selecionados para monitoramento.

Implementar pesquisa, ordenação e paginação.

Evitar downloads desnecessários do conteúdo das planilhas durante a descoberta.

## 6. REVISÃO DE ESCOPO OAUTH

Audite os escopos Google configurados na Sprint 02.

Verifique se permitem listar amplamente as planilhas acessíveis à conta.

Não presuma que drive.file concede acesso à listagem geral do Drive.

Se necessário, implemente autorização incremental para um escopo que permita a descoberta pretendida, priorizando a menor permissão efetivamente suficiente.

Documente:

- Escopo escolhido.
- Justificativa.
- Impacto sobre privacidade.
- Limitações da descoberta.
- Eventual necessidade de verificação OAuth.
- Requisitos de publicação para usuários externos.

Não amplie permissões silenciosamente.

A autorização deverá ocorrer pelo fluxo oficial de consentimento Google.

Não implemente um servidor próprio para contornar restrições OAuth.

Não considere o aplicativo publicável para qualquer conta Google apenas por funcionar com usuários de teste.

## 7. CADASTRO E GERENCIAMENTO

Implemente os seguintes recursos:

- Adicionar planilha pela descoberta do Drive.
- Adicionar planilha por URL.
- Validar URL.
- Evitar duplicidade de spreadsheetId por conta.
- Exibir nome da planilha.
- Exibir conta Google associada.
- Exibir situação da autorização.
- Exibir status de integridade.
- Exibir última verificação.
- Pausar monitoramento.
- Retomar monitoramento.
- Remover cadastro.
- Confirmar exclusão de cadastro.
- Preservar ou excluir histórico conforme escolha explícita.
- Atualizar metadados do arquivo.
- Abrir a planilha no navegador.
- Exibir diagnóstico de erro de acesso.

Imponha o limite de 30 planilhas cadastradas para monitoramento ativo.

A listagem de arquivos descobertos não deverá estar limitada a 30.

Não cadastre automaticamente todos os arquivos encontrados sem uma preferência explícita do usuário.

## 8. DASHBOARD

Implemente uma interface principal com:

- Total de planilhas cadastradas.
- Total de planilhas Online.
- Total de planilhas Offline.
- Total de planilhas Indeterminadas.
- Total de planilhas pausadas.
- Estado da conexão Google.
- Conta autenticada.
- Últimas verificações.
- Últimas alterações.
- Falhas estruturais recentes.
- Acesso rápido aos diagnósticos.

Os indicadores deverão consumir dados reais do backend.

Enquanto não houver uma verificação, exiba "Ainda não verificada".

Não apresente "Online" como estado padrão de uma planilha recém-cadastrada.

Não utilize dados simulados fora de ambientes explicitamente demonstrativos ou de testes.

## 9. NAVEGAÇÃO

Implemente navegação entre:

1. Visão geral.
2. Planilhas.
3. Detalhes de planilha.
4. Atividades.
5. Regras e baselines.
6. Configurações.

A navegação deverá ser rápida, consistente e acessível.

Utilize React, TypeScript, Tailwind e shadcn/ui.

Implemente estados vazios, carregamento, erros e feedback das operações.

O aplicativo deverá adaptar seu layout a diferentes tamanhos de janela no Windows 11.

## 10. SELEÇÃO DE ABAS

Na tela de detalhes da planilha:

1. Liste todas as abas disponíveis.
2. Exiba título e identificador de cada aba.
3. Permita selecionar todas as abas.
4. Permita selecionar abas individualmente.
5. Permita excluir abas do monitoramento.
6. Persista as preferências no SQLite.
7. Preserve as seleções após reiniciar o aplicativo.
8. Trate abas removidas ou renomeadas.
9. Exiba advertências sobre dependências entre abas.
10. Exija uma configuração de integridade válida antes de iniciar monitoramento.

Utilize sheetId como referência de identidade da aba sempre que aplicável.

O nome visual não deverá ser usado como único identificador.

## 11. CONFIGURAÇÃO DE REGRAS

Implemente interface para configurar:

- Colunas obrigatórias.
- Identidade e ordem de colunas.
- Cabeçalhos protegidos.
- Fórmulas críticas.
- Regiões protegidas.
- Abas obrigatórias.
- Regras de renomeação de abas.
- Dependências entre abas.
- Dependências Apps Script.
- Colunas-chave para identificar registros.
- Regras de severidade.
- Dependências críticas e opcionais.

As regras deverão ser baseadas nos contratos do motor de integridade da Sprint 03.

Não replique a lógica de comparação no frontend.

Ofereça explicações compreensíveis para usuários não programadores.

## 12. BASELINES E APROVAÇÃO

Implemente um fluxo visual completo para referências estruturais.

Fluxo:

1. Capturar snapshot.
2. Exibir estrutura identificada.
3. Apresentar regras propostas.
4. Permitir revisar e ajustar as regras.
5. Solicitar aprovação explícita.
6. Registrar baseline versionado.
7. Definir baseline ativo.
8. Exibir versões anteriores.
9. Comparar uma versão com outra.
10. Permitir aprovar nova versão após alteração legítima.

Nunca atualize um baseline automaticamente.

Nunca substitua uma referência aprovada apenas porque o estado atual mudou.

Uma planilha sem baseline aprovado deverá aparecer como "Configuração pendente", não como Online.

## 13. APPS SCRIPT — INTERFACE

Implemente uma área de scripts associados a cada planilha.

Exibir:

- Projetos Apps Script identificados ou cadastrados.
- scriptId quando disponível.
- Nome do projeto.
- Estado de autorização.
- Data da última consulta.
- Arquivos de código acessíveis.
- Alterações de código detectadas.
- Dependências reconhecidas.
- Dependências não verificáveis.
- Problemas estruturais encontrados.
- Cobertura de análise.

Permita cadastrar manualmente um scriptId quando a descoberta automática não for viável.

Não afirme que todos os projetos vinculados são descobertos automaticamente.

Não execute nem edite scripts.

## 14. HISTÓRICO DE ATIVIDADES

Implemente uma linha do tempo com:

- Data e horário da detecção.
- Nome da planilha.
- Nome da aba.
- Tipo de alteração.
- Categoria.
- Severidade.
- Estado resultante.
- Diagnóstico.
- Identificador da verificação.
- Regra afetada.
- Informações anteriores e atuais, quando disponíveis.

Implemente filtros por:

- Planilha.
- Aba.
- Data.
- Tipo de alteração.
- Severidade.
- Status.
- Apps Script.
- Alteração permitida ou violação estrutural.

Implemente pesquisa e paginação.

Evite carregar todo o histórico em memória.

Não atribua a autoria de uma alteração a uma pessoa sem dados confiáveis que sustentem essa identificação.

## 15. DETALHES DE DIAGNÓSTICO

Ao abrir um evento, apresente:

- O que foi alterado.
- Onde ocorreu a alteração.
- Estrutura esperada.
- Estrutura encontrada.
- Qual regra foi violada.
- Impacto conhecido.
- Dependências afetadas.
- Status de integridade.
- Possíveis ações de correção.

Não execute correções automaticamente.

As orientações de correção deverão ser informativas.

Quando houver incerteza, exiba explicitamente "Não foi possível determinar".

## 16. CONFIGURAÇÕES

Implemente uma área de preferências contendo:

- Conta Google conectada.
- Desconectar conta.
- Reautorizar permissões.
- Estado das APIs.
- Intervalo de monitoramento.
- Ativação de notificações.
- Política de histórico.
- Preferência de inicialização com Windows.
- Preferência de execução minimizada.
- Preferências de aparência.
- Informações da versão do aplicativo.

As configurações de intervalo e notificações deverão ser persistidas.

O agendador e as notificações definitivas serão implementados na Sprint 05.

Não apresente controles não funcionais como se estivessem ativos.

Se alguma configuração depender de módulo futuro, sinalize que ficará disponível posteriormente.

## 17. PERSISTÊNCIA

Integre os componentes às tabelas SQLite existentes.

Crie migrações somente quando necessárias.

Garanta persistência para:

- Planilhas cadastradas.
- Abas monitoradas.
- Regras de integridade.
- Baselines.
- Associações de Apps Script.
- Preferências.
- Histórico.
- Estado da última verificação.

Utilize operações transacionais para alterações críticas.

Implemente tratamento de duplicidades e falhas de persistência.

## 18. COMANDOS TAURI

Implemente ou adapte comandos tipados para:

- discover_spreadsheets
- list_registered_spreadsheets
- register_spreadsheet
- remove_spreadsheet
- pause_spreadsheet
- resume_spreadsheet
- list_spreadsheet_tabs
- update_monitored_tabs
- list_integrity_rules
- update_integrity_rules
- preview_baseline
- approve_baseline
- list_baseline_versions
- get_spreadsheet_diagnostics
- list_change_events
- list_associated_scripts
- get_dashboard_summary

Utilize os contratos já existentes quando possível.

Não crie comandos duplicados para operações equivalentes.

## 19. EXPERIÊNCIA DO USUÁRIO

Priorize:

- Interface limpa.
- Navegação intuitiva.
- Textos em português brasileiro.
- Indicadores de integridade compreensíveis.
- Visualização clara de erros.
- Carregamento progressivo.
- Tabelas com filtros.
- Busca rápida.
- Confirmação de operações destrutivas.
- Acessibilidade de teclado.
- Feedback visual consistente.

Evite excesso de modais e interrupções.

Não exponha detalhes técnicos desnecessários ao usuário comum, mas permita consultar relatórios avançados.

## 20. TESTES UNITÁRIOS E DE COMPONENTES

Crie testes para:

1. Login visual conectado/desconectado.
2. Descoberta com resultados.
3. Descoberta sem resultados.
4. Descoberta com permissão insuficiente.
5. Paginação de arquivos.
6. Arquivos compartilhados.
7. Cadastro por URL válida.
8. URL inválida.
9. Planilha duplicada.
10. Limite de 30 cadastros.
11. Seleção de abas.
12. Remoção de cadastro.
13. Persistência de regras.
14. Criação de baseline.
15. Tentativa de substituição não aprovada.
16. Exibição de Online.
17. Exibição de Offline.
18. Exibição de Indeterminado.
19. Planilha sem baseline.
20. Filtros do histórico.
21. Visualização de dependências Apps Script.
22. Estados de carregamento e erro.
23. Recuperação de dados após reinício.
24. Compatibilidade dos contratos Tauri.

Utilize mocks somente nos testes apropriados.

Não apresente fixtures simuladas como dados reais.

## 21. TESTES DE INTEGRAÇÃO REAL

Com conta Google autorizada:

1. Autenticar no aplicativo.
2. Descobrir planilhas acessíveis.
3. Navegar entre páginas de resultados.
4. Cadastrar uma planilha descoberta.
5. Cadastrar outra por URL.
6. Listar abas reais.
7. Selecionar abas monitoradas.
8. Capturar estrutura real.
9. Aprovar baseline.
10. Executar análise sob demanda.
11. Consultar relatório de integridade.
12. Consultar histórico.
13. Associar Apps Script autorizado.
14. Reiniciar o aplicativo.
15. Confirmar persistência.

Não faça alterações destrutivas em arquivos de produção.

Se testes reais estiverem indisponíveis, registre a pendência.

## 22. DOCUMENTAÇÃO

Atualize:

- README.md
- docs/interface.md
- docs/spreadsheet-discovery.md
- docs/spreadsheet-management.md
- docs/baseline-ui.md
- docs/integrity-rules-ui.md
- docs/apps-script-ui.md
- docs/testing.md
- docs/oauth-setup.md

Documente especialmente:

- Como o usuário faz login.
- Como encontra planilhas.
- Como cadastra por URL.
- Como escolhe abas.
- Como aprova baseline.
- Como interpreta os estados.
- Como consulta alterações.
- Como associa um Apps Script.
- Quais permissões Google são necessárias.
- Quais etapas de publicação OAuth permanecem pendentes.

## 23. CRITÉRIOS DE ACEITE

A sprint somente estará concluída quando:

1. O dashboard consumir dados reais do backend.
2. A conta Google conectada for identificada.
3. A descoberta automática funcionar com escopos adequados.
4. O cadastro por URL funcionar.
5. O limite de 30 planilhas for respeitado.
6. A seleção de abas funcionar.
7. As regras de proteção puderem ser configuradas.
8. Os baselines puderem ser aprovados visualmente.
9. Os diagnósticos da Sprint 03 puderem ser consultados.
10. O histórico apresentar eventos persistidos.
11. Os projetos Apps Script associados puderem ser consultados.
12. As preferências forem persistidas.
13. A interface funcionar no Windows 11.
14. Os testes automatizados passarem.
15. O projeto compilar.
16. A documentação estiver atualizada.

Não considere aprovada uma funcionalidade cuja implementação depende de APIs não autorizadas e não testadas.

## 24. RELATÓRIO FINAL

Ao terminar, apresente:

- Funcionalidades implementadas.
- Telas concluídas.
- Arquivos modificados.
- Migrações criadas.
- Contratos Tauri implementados.
- Testes automatizados e resultados.
- Testes reais e evidências.
- Permissões OAuth necessárias.
- Pendências de publicação Google.
- Intervenções humanas indispensáveis.
- Limitações técnicas.
- Preparação para a Sprint 05.

## 25. RESTRIÇÃO DE ESCOPO

Execute exclusivamente a Sprint 04.

Não implemente antecipadamente:

- Serviço definitivo de monitoramento periódico.
- Execução contínua na bandeja.
- Notificações nativas definitivas.
- Serviço de inicialização automática com Windows.
- Instalador final de distribuição.
- Atualização automática do aplicativo.

Essas funcionalidades pertencem às Sprints 05 e 06.

Ao concluir, encerre a execução e aguarde o Prompt 05.

## 4. Ações humanas necessárias

A Sprint 04 exige pouca intervenção se as APIs já estiverem configuradas e a sessão Google estiver autorizada.

| Ação                              | Responsável | Necessidade                              |
| --------------------------------- | ----------- | ---------------------------------------- |
| Implementar todas as telas        | Codex       | Sem ação humana                          |
| Implementar descoberta pelo Drive | Codex       | Sem ação humana, se já autorizado        |
| Ajustar escopos OAuth             | Codex       | Novo consentimento Google, se necessário |
| Cadastrar planilhas fictícias     | Codex       | Sem ação humana, com acesso autorizado   |
| Configurar regras sintéticas      | Codex       | Sem ação humana                          |
| Aprovar baseline de teste         | Codex       | Automatizável em ambiente de teste       |
| Aprovar baseline de produção      | Usuário     | Necessário                               |
| Verificar interface no Windows 11 | Codex       | Automatizável com acesso ao desktop      |
| Validar experiência visual final  | Usuário     | Recomendado                              |

O principal possível bloqueio será a ampliação das permissões Google. A descoberta de todas as planilhas acessíveis não deve depender de um usuário inserir URLs ou de informar projetos Cloud, mas pode exigir um escopo do Drive sujeito à verificação do Google para distribuição pública.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers



## 5. Gate de aprovação da Sprint 04

Checklist de homologação

0/12

Login Google e identificação da conta

Descoberta automática de planilhas autorizadas

Cadastro manual por link

Seleção de até 30 planilhas

Seleção individual de abas

Configuração de regras de integridade

Aprovação visual do baseline

Visualização dos resultados de integridade

Histórico e diagnósticos

Associação de Apps Script

Persistência após reinicialização

Testes e build aprovados

&#x20;Copiar checklist

## 6. Resultado esperado

Ao término da Sprint 04, o Sheets Monitor deverá permitir que um usuário:

1. Faça login com sua conta Google autorizada.
2. Encontre as planilhas às quais o aplicativo tem permissão de acesso.
3. Selecione até 30 arquivos.
4. Configure as abas e regras de proteção.
5. Aprove a estrutura inicial.
6. Execute verificações sob demanda e visualize os diagnósticos.

A principal diferença entre as Sprints 04 e 05 é a automação do monitoramento. Na Sprint 04, o aplicativo já deverá permitir executar e consultar análises de integridade. Na Sprint 05, essas análises passarão a ser executadas automaticamente em intervalos definidos, inclusive com a janela minimizada, emitindo notificações nativas do Windows.

Assim, a Sprint 04 fecha a experiência de gerenciamento, e a Sprint 05 a transforma em um sistema de vigilância contínua.