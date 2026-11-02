# Sheets Monitor — Sprint 03: Motor de Integridade Estrutural

ETAPA 03 DE 06 · PROMPT PARA CODEX · WINDOWS 11

A Sprint 03 será responsável pelo componente mais importante do aplicativo: o motor que identifica modificações nas planilhas, diferencia alterações permitidas de falhas estruturais e verifica se as dependências de Apps Script continuam válidas.

Ao final desta etapa, o Sheets Monitor deverá conseguir analisar planilhas reais e classificá-las como Online, Offline ou Indeterminado, mesmo que o dashboard definitivo e o monitoramento periódico sejam implementados nas sprints posteriores.

Sprint

## 03 de 06

Backlog

## SM-012 a SM-017

Prioridade

## Crítica

Estimativa

## 6–10 dias úteis

## 1. Entregas obrigatórias

| ID     | Funcionalidade           | Critério de aceite                                         |
| ------ | ------------------------ | ---------------------------------------------------------- |
| SM-012 | Baseline estrutural      | Criar, armazenar e versionar referências aprovadas         |
| SM-013 | Comparação estrutural    | Detectar mudanças de abas, colunas e cabeçalhos            |
| SM-014 | Integridade de fórmulas  | Reconhecer fórmulas protegidas e erros                     |
| SM-015 | Dependências Apps Script | Analisar referências estáticas e verificar compatibilidade |
| SM-016 | Classificação            | Produzir status e diagnóstico fundamentado                 |
| SM-017 | Detecção de alterações   | Registrar diferenças de valores e linhas sem duplicações   |

### Regras inegociáveis

- Alterações de linhas e valores devem ser permitidas, desde que não violem estruturas protegidas.
- Reordenação, exclusão ou alteração de colunas protegidas devem ser detectadas.
- Fórmulas críticas não podem desaparecer nem perder sua funcionalidade.
- Dependências Apps Script devem ser verificadas conforme sua cobertura técnica.
- A referência aprovada não pode ser substituída automaticamente.
- Todo evento deve ser identificável, rastreável e passível de diagnóstico.

Distinção fundamental: uma alteração permitida produz um evento, mas não uma falha. Já uma violação estrutural produz um evento e altera o status para Offline.

## 2. Arquitetura do motor de integridade

Coleta de estados

Google Sheets API + Apps Script API

Normalização dos dados

Identificação estável de abas, colunas, fórmulas e dependências

Baseline

Estrutura aprovada

Snapshot atual

Estrutura observada

Mecanismo de comparação

Diff estrutural + diff de conteúdo + análise de dependências

Classificação

Online / Offline / Indeterminado

Eventos

Alterações e diagnóstico

A implementação deverá ser concentrada no backend Rust. O frontend consumirá resultados tipados, sem reproduzir a lógica de integridade no React.

A coleta usará leituras direcionadas da Sheets API, incluindo `spreadsheets.get` e `values.batchGet` com `valueRenderOption=FORMULA`, conforme o tipo de dado. A API permite selecionar campos e intervalos para reduzir requisições e volume transferido.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers+1



## 3. Regras de comparação

| Alteração identificada                 | Resultado esperado                                       |
| -------------------------------------- | -------------------------------------------------------- |
| Novo registro em linha                 | Evento informativo, Online                               |
| Linha excluída                         | Evento informativo, Online                               |
| Valor de célula alterado               | Evento informativo, Online                               |
| Cabeçalho obrigatório renomeado        | Violação, Offline                                        |
| Colunas protegidas invertidas          | Violação, Offline                                        |
| Fórmula crítica apagada                | Violação, Offline                                        |
| Fórmula ajustada por inserção de linha | Online, se semanticamente compatível                     |
| Erro `#REF!` em fórmula protegida      | Violação, Offline                                        |
| Aba monitorada excluída                | Violação, Offline                                        |
| Dependência Apps Script inválida       | Violação, Offline                                        |
| Dependência dinâmica não verificável   | Cobertura parcial ou Indeterminado, conforme criticidade |

### Como evitar falsos positivos

O Codex não deverá utilizar apenas comparação literal de coordenadas e fórmulas.

Por exemplo, ao inserir uma linha acima de uma fórmula, o Google Sheets pode deslocar a célula e ajustar suas referências automaticamente. Isso não significa necessariamente que a estrutura foi quebrada.

O motor deverá normalizar fórmulas e acompanhar regiões protegidas, usando referências relativas quando tecnicamente possível. Nos casos ambíguos, deverá produzir um diagnóstico de incerteza em vez de afirmar que a fórmula foi quebrada.

### Regra de identificação de linhas

A detecção de inserções, exclusões e edições será preferencialmente baseada em uma coluna-chave configurada para cada região de dados.

Quando não existir uma chave estável, o sistema utilizará alinhamento e comparação de sequências de linhas, indicando que a classificação da operação pode ser ambígua.

Assim, poderá registrar com precisão que o conteúdo mudou sem inventar se determinada linha foi inserida, excluída ou editada.

# 4. Prompt 03 — Implementação integral no Codex

Copie o prompt a seguir para o Codex após a Sprint 02. Ele foi elaborado para permitir máxima autonomia dentro desta sprint, sem autorizar o desenvolvimento antecipado das etapas 04, 05 e 06.

# PROMPT 03 — Motor de Integridade Estrutural do Sheets Monitor

Você é o agente principal de engenharia responsável pela Sprint 03 do Sheets Monitor.

Execute integralmente esta sprint, respeitando o contrato de autonomia estabelecido no Prompt 00 e preservando as implementações homologadas nas Sprints 01 e 02.

## 1. CONTEXTO

O Sheets Monitor é um aplicativo desktop para Windows 11, desenvolvido com Tauri 2, React, TypeScript, Rust e SQLite.

O aplicativo deverá monitorar até 30 planilhas Google Sheets, incluindo:

- Estrutura de abas e colunas.
- Fórmulas críticas.
- Alterações em linhas e valores.
- Dependências de projetos Google Apps Script.
- Histórico e classificação de integridade.

A Sprint 02 implementou as integrações com Google Sheets API, Google Drive API e Google Apps Script API.

Sua tarefa agora é desenvolver o motor completo de integridade que será utilizado pelas futuras interfaces e pelo agendador de monitoramento.

## 2. AUTONOMIA OPERACIONAL

Você está autorizado a:

1. Inspecionar integralmente o repositório.
2. Consultar documentação técnica oficial.
3. Criar, modificar e organizar arquivos.
4. Instalar dependências necessárias e seguras.
5. Implementar algoritmos em Rust.
6. Desenvolver testes automatizados.
7. Executar verificações com planilhas de teste autorizadas.
8. Corrigir falhas identificadas.
9. Refatorar módulos existentes quando necessário.
10. Criar migrações SQLite.
11. Atualizar documentação.
12. Criar commits em branch de desenvolvimento.

Não solicite confirmação para decisões técnicas rotineiras e reversíveis.

Interrompa apenas operações que dependam de autorização humana indispensável.

Não modifique planilhas reais de produção.

Não atualize automaticamente referências estruturais aprovadas.

## 3. AUDITORIA DAS SPRINTS ANTERIORES

Antes de implementar:

1. Leia AGENTS.md.
2. Analise README.md.
3. Inspecione os módulos Rust existentes.
4. Revise os modelos TypeScript.
5. Inspecione as migrações SQLite.
6. Verifique os contratos das APIs Google.
7. Execute os testes existentes.
8. Identifique problemas de integração.

Preserve o funcionamento das Sprints 01 e 02.

Não recrie módulos já funcionais sem justificativa técnica.

## 4. MODELO DE SNAPSHOT

Crie uma representação normalizada do estado observado de cada planilha.

O snapshot deve armazenar, conforme o escopo configurado:

- spreadsheetId.
- Identificação da conta autorizada.
- Data e hora da captura.
- Identificadores estáveis das abas.
- Títulos e posições das abas.
- Informações sobre linhas e colunas.
- Cabeçalhos monitorados.
- Identificadores de colunas definidos pelo monitor.
- Intervalos protegidos.
- Fórmulas críticas.
- Valores e erros de fórmulas.
- Referências entre abas.
- Impressões digitais de conteúdo.
- Projetos Apps Script associados.
- Hashes dos arquivos de script acessíveis.
- Dependências identificadas.
- Cobertura e limitações da captura.

Utilize modelos fortemente tipados em Rust.

Implemente serialização e desserialização versionadas.

Não armazene desnecessariamente cópias integrais dos dados das planilhas.

## 5. BASELINE E VERSIONAMENTO

Implemente o conceito de baseline estrutural.

O baseline representa a estrutura aprovada de uma planilha e será utilizado para verificações futuras.

Requisitos:

1. Criar uma referência a partir de um snapshot válido.
2. Armazenar a referência no SQLite.
3. Associar a referência à planilha e às abas monitoradas.
4. Registrar versão, horário e origem da aprovação.
5. Permitir consultar versões anteriores.
6. Garantir que apenas uma versão esteja ativa por escopo monitorado.
7. Impedir substituição automática de baseline.
8. Preparar uma operação explícita de aprovação de nova versão.
9. Preservar rastreabilidade de todas as versões.

Durante a Sprint 03, a aprovação poderá ser disponibilizada por comando de desenvolvimento ou interface técnica simples.

Não implemente ainda o fluxo visual definitivo de aprovação.

## 6. COMPARAÇÃO ESTRUTURAL

Implemente um módulo de comparação entre baseline e snapshot atual.

Deverá identificar:

- Exclusão de abas monitoradas.
- Alteração de nomes quando protegidos.
- Reordenação de abas quando configurada como crítica.
- Exclusão de colunas protegidas.
- Inserção incompatível de colunas.
- Reordenação de colunas protegidas.
- Alteração de cabeçalhos protegidos.
- Deslocamento de intervalos estruturais.
- Remoção de regiões obrigatórias.
- Alterações incompatíveis em intervalos nomeados, quando monitorados.

Utilize sheetId para identificar abas, evitando depender exclusivamente do título.

Para colunas, implemente identidade lógica com base em regras explícitas, cabeçalhos, posição e outros metadados disponíveis.

Não presuma que letras de coluna representam identidades imutáveis.

Ao identificar uma violação, gere um diagnóstico contendo:

- Planilha afetada.
- Aba afetada.
- Regra violada.
- Estrutura esperada.
- Estrutura encontrada.
- Severidade.
- Horário da detecção.
- Informações suficientes para localizar o problema.

## 7. ALTERAÇÕES PERMITIDAS

O sistema deve permitir:

- Inserção de linhas de dados.
- Exclusão de linhas de dados.
- Edição de valores em linhas de dados.

Essas alterações não devem modificar o status para Offline quando preservarem as regras estruturais.

Entretanto, toda diferença detectada deverá gerar um evento informativo.

Implemente comparação de dados com suporte a:

- Identificadores estáveis de registros quando disponíveis.
- Colunas-chave configuráveis.
- Detecção de valores modificados.
- Detecção de novos registros.
- Detecção de registros removidos.
- Comparação por blocos ou hashes para eficiência.

Quando não for possível determinar com segurança o tipo exato de edição, classifique a ocorrência como alteração de conteúdo não especificada.

Não invente uma operação de inserção, exclusão ou edição.

## 8. FÓRMULAS PROTEGIDAS

Implemente um módulo especializado em fórmulas.

Requisitos:

1. Identificar fórmulas em regiões monitoradas.
2. Registrar fórmulas críticas no baseline.
3. Detectar remoção de fórmulas.
4. Detectar substituição indevida por valores.
5. Detectar mudanças não autorizadas em fórmulas protegidas.
6. Identificar erros como #REF!, #NAME?, #DIV/0! e outros.
7. Distinguir alteração textual de incompatibilidade estrutural.
8. Normalizar referências relativas quando tecnicamente seguro.
9. Considerar deslocamentos válidos decorrentes de inserção e exclusão de linhas.
10. Verificar referências entre abas.

Implemente pelo menos dois modos de proteção:

MODO EXATO:\
A fórmula deve corresponder à referência aprovada, respeitando regras explícitas de deslocamento.

MODO ESTRUTURAL:\
A fórmula deverá preservar sua lógica e dependências, permitindo transformações consideradas equivalentes pelo algoritmo.

Não afirme equivalência sem comprovação.

Quando a equivalência não puder ser demonstrada, registre uma análise inconclusiva.

Não utilize IA generativa como única fonte para decidir se uma fórmula é válida.

O resultado deverá ser baseado em regras determinísticas, com testes automatizados.

## 9. MONITORAMENTO DE APPS SCRIPT

Utilize o módulo criado na Sprint 02 para consultar os projetos Apps Script autorizados.

Implemente:

1. Captura e armazenamento de hashes do código.
2. Comparação entre a versão atual e a versão de referência.
3. Identificação de arquivos adicionados.
4. Identificação de arquivos removidos.
5. Identificação de alterações no código.
6. Identificação de dependências estáticas.
7. Validação de dependências estruturais.
8. Relatórios de cobertura da análise.

Reconheça inicialmente padrões como:

- getSheetByName("Nome")
- getRange("A1")
- getRange(linha, coluna, quantidadeLinhas, quantidadeColunas)
- getColumn()
- getLastRow()
- getLastColumn()
- getValues()
- getDisplayValues()
- getFormulas()

Implemente extração de dependências com analisador sintático de JavaScript, quando adequado.

Não dependa exclusivamente de expressões regulares para interpretar código complexo.

Diferencie os seguintes casos:

A. Referência estática verificável.\
B. Referência parcialmente verificável.\
C. Referência dinâmica não verificável.\
D. Dependência estrutural comprovadamente inválida.

Uma alteração no texto do script deve gerar evento, mas somente uma violação crítica confirmada deve produzir status Offline.

Não execute, edite ou publique scripts.

Não presuma que todas as dependências dinâmicas podem ser descobertas.

## 10. CLASSIFICAÇÃO DE INTEGRIDADE

Implemente a classificação por aba e por planilha.

Estados:

ONLINE:\
Todas as verificações críticas obrigatórias foram concluídas com sucesso.

OFFLINE:\
Existe uma violação crítica comprovada.

INDETERMINADO:\
Não foi possível concluir uma verificação crítica obrigatória.

VERIFICANDO:\
Uma análise está em execução.

O estado OFFLINE deverá ter precedência sobre o estado INDETERMINADO quando houver uma violação crítica comprovada, sem ocultar as verificações inconclusivas.

A agregação por planilha deverá considerar apenas abas monitoradas e dependências críticas associadas.

Abas fora do escopo não deverão gerar falhas, exceto quando afetarem dependências explicitamente protegidas.

Separe o estado de conectividade do estado de integridade.

## 11. MOTOR DE DIFERENÇAS E EVENTOS

Implemente um motor que gere eventos a partir das diferenças observadas.

Cada evento deverá conter:

- eventId.
- spreadsheetId.
- sheetId, se aplicável.
- Tipo de alteração.
- Categoria.
- Severidade.
- Referência anterior e atual, quando cabível.
- Data e hora da detecção.
- Identificador da verificação.
- Identificador da regra violada, se houver.
- Informações necessárias ao diagnóstico.

Implemente deduplicação.

Um problema persistente não deverá criar um novo evento idêntico em todas as verificações.

Uma alteração nova ou a recuperação de uma falha deverá produzir um evento distinto.

Preserve dados suficientes para a implementação posterior de notificações nativas.

Não implemente ainda o agendador contínuo ou o serviço definitivo de notificações.

## 12. PERSISTÊNCIA SQLITE

Implemente ou evolua as tabelas necessárias:

- baselines
- baseline_versions
- structural_snapshots
- integrity_rules
- monitored_sheets
- script_projects
- script_dependencies
- monitoring_checks
- change_events

Utilize migrações versionadas.

Implemente transações para garantir consistência.

Adicione índices para consultas por planilha, aba, data, severidade e estado.

Evite duplicar grandes volumes de valores sem necessidade.

Implemente retenção configurável para snapshots e histórico, preservando as referências aprovadas.

## 13. CONTRATOS E COMANDOS TAURI

Implemente contratos tipados para operações como:

- capture_snapshot
- create_baseline
- list_baselines
- get_active_baseline
- compare_snapshot
- validate_integrity
- analyze_script_dependencies
- list_integrity_events
- get_integrity_report

Adapte os nomes às convenções existentes, mantendo documentação equivalente.

Garanta validação de entradas e tratamento estruturado de erros.

Os contratos devem permitir integração posterior com o dashboard.

## 14. TESTES UNITÁRIOS

Crie testes obrigatórios para:

1. Inserção de linha permitida.
2. Exclusão de linha permitida.
3. Edição de valor permitida.
4. Reordenação de colunas.
5. Exclusão de coluna protegida.
6. Alteração de cabeçalho crítico.
7. Exclusão de aba monitorada.
8. Renomeação de aba protegida.
9. Fórmula crítica removida.
10. Fórmula substituída por valor.
11. Fórmula com referência quebrada.
12. Fórmula validamente deslocada por inserção de linha.
13. Fórmula validamente deslocada por exclusão de linha.
14. Dependência Apps Script válida.
15. Dependência Apps Script inválida.
16. Referência dinâmica não verificável.
17. Código Apps Script alterado.
18. Falha de autenticação.
19. Falha de conexão.
20. Violação persistente sem duplicação de eventos.
21. Recuperação de falha estrutural.
22. Baseline anterior preservado.
23. Tentativa de atualização automática indevida do baseline.
24. Classificação com múltiplas abas.
25. Conflito entre violação confirmada e análise inconclusiva.

Utilize fixtures representativas de planilhas e scripts reais.

Os testes devem ser determinísticos e reproduzíveis.

## 15. TESTES DE INTEGRAÇÃO

Com recursos de teste previamente autorizados:

1. Capturar snapshot de planilha real.
2. Criar baseline inicial.
3. Realizar uma alteração permitida.
4. Verificar que o status continua Online.
5. Realizar uma alteração estrutural proibida.
6. Verificar que o status muda para Offline.
7. Corrigir a alteração.
8. Verificar recuperação do status Online.
9. Alterar um Apps Script de teste.
10. Detectar a mudança no código.
11. Invalidar uma dependência estrutural de teste.
12. Verificar o diagnóstico correspondente.

Não modifique planilhas de produção.

Se não houver recursos autorizados para testes destrutivos, utilize fixtures locais e registre a homologação real como pendente.

## 16. EFICIÊNCIA E ESCALABILIDADE

Projete o motor para até 30 planilhas cadastradas.

Utilize:

- Consultas direcionadas.
- Processamento incremental quando aplicável.
- Hashes de conteúdo.
- Comparação por blocos.
- Cache local de metadados.
- Estruturas de dados eficientes.
- Limites de memória.
- Logs de desempenho.

Não implemente otimizações que prejudiquem a precisão da comparação.

Documente limitações de arquivos muito grandes.

Não execute verificações contínuas nesta sprint; ofereça operações de análise sob demanda para testes.

## 17. DOCUMENTAÇÃO

Atualize:

- docs/integrity-engine.md
- docs/baselines.md
- docs/formula-analysis.md
- docs/apps-script-dependencies.md
- docs/event-model.md
- docs/testing.md
- README.md

Documente exemplos de:

- Estrutura válida.
- Alteração permitida.
- Violação estrutural.
- Fórmula quebrada.
- Dependência inválida.
- Análise inconclusiva.
- Recuperação de integridade.

## 18. CRITÉRIOS DE ACEITE

A Sprint 03 será concluída apenas quando:

1. Snapshots estruturais puderem ser capturados.
2. Baselines puderem ser criados e versionados.
3. O motor identificar violações estruturais.
4. Alterações permitidas não gerarem Offline indevidamente.
5. Fórmulas críticas forem monitoradas.
6. Dependências Apps Script forem analisadas dentro da cobertura declarada.
7. Os estados forem classificados corretamente.
8. Os eventos forem persistidos sem duplicações indevidas.
9. Os testes automatizados passarem.
10. A integração existente permanecer funcional.
11. O aplicativo compilar corretamente.
12. As limitações de análise forem documentadas.

Não declare uma integração real validada caso apenas testes simulados tenham sido executados.

## 19. RELATÓRIO FINAL

Apresente:

- Funcionalidades implementadas.
- Arquivos criados e modificados.
- Algoritmos utilizados.
- Regras de integridade implementadas.
- Tipos de dependências Apps Script reconhecidos.
- Cobertura da análise.
- Resultados dos testes.
- Falhas corrigidas.
- Pendências técnicas.
- Intervenções humanas necessárias.
- Preparação para a Sprint 04.

## 20. RESTRIÇÃO FINAL

Execute exclusivamente a Sprint 03.

Não implemente antecipadamente:

- Dashboard definitivo.
- Interface completa de gerenciamento de planilhas.
- Histórico visual definitivo.
- Serviço de monitoramento contínuo.
- Notificações nativas definitivas.
- Instalador de produção.

Esses componentes pertencem às Sprints 04, 05 e 06.

Ao concluir esta etapa, apresente o relatório, finalize suas operações e aguarde o Prompt 04.

# 5. Testes de homologação da Sprint 03

O Codex deverá executar testes automatizados e, quando houver autorização, testes de integração com planilhas fictícias.

Checklist de integridade

0/12 validados

Inserir linha de dados

Online

Excluir linha de dados

Online

Editar valor comum

Online

Inverter colunas protegidas

Offline

Remover coluna obrigatória

Offline

Apagar fórmula crítica

Offline

Deslocar fórmula sem quebrá-la

Online

Excluir aba monitorada

Offline

Invalidar referência de Apps Script

Offline

Encontrar referência dinâmica crítica sem validação

Indeterminado

Perder acesso a uma planilha

Indeterminado

Corrigir violação estrutural

Online

&#x20;Copiar checklist

Os estados apresentados são os resultados esperados sob regras de proteção previamente configuradas. Marcar itens aqui não executa testes no repositório.

# 6. Intervenções humanas necessárias

Esta sprint foi planejada para exigir menos intervenção humana que a Sprint 02, pois as integrações Google já deverão estar implementadas.

| Ação                                     | Necessidade               | Automação possível                 |
| ---------------------------------------- | ------------------------- | ---------------------------------- |
| Disponibilizar planilhas fictícias       | Sim                       | Codex pode criar fixtures locais   |
| Aprovar a primeira referência estrutural | Sim, para planilhas reais | IA pode propor a referência        |
| Disponibilizar Apps Script de teste      | Sim                       | Codex pode gerar scripts fictícios |
| Executar testes de código                | Não                       | Totalmente automatizável           |
| Comparar estruturas                      | Não                       | Totalmente automatizável           |
| Validar fórmulas                         | Não, nos casos cobertos   | Automatizável                      |
| Confirmar regras ambíguas                | Eventual                  | IA apresenta diagnóstico           |
| Validar resultados reais                 | Recomendado               | IA fornece evidências              |

O Codex deverá criar automaticamente uma biblioteca de cenários de teste, incluindo uma planilha com múltiplas abas e scripts com dependências conhecidas.

Para diminuir a participação humana, a IA poderá utilizar baselines sintéticos aprovados previamente por especificação durante o desenvolvimento. A aprovação humana ficará reservada para quando o aplicativo passar a monitorar planilhas reais.

## 7. Ponto de atenção: fórmulas e Apps Script

O maior risco desta sprint é o Codex produzir um sistema que aparentemente identifica falhas, mas gera falsos positivos ao interpretar mudanças legítimas.

Exemplo: uma planilha possui dados nas colunas A, B e C, e um script executa:

```
const aba = ss.getSheetByName("Orç 2026");
const dados = aba.getRange(6, 1, 20, 3).getValues();
```

O método continua executável mesmo que as colunas sejam reordenadas. Entretanto, os dados retornados podem deixar de corresponder à estrutura esperada.

O motor deverá comparar não apenas se `getRange()` retorna resultados, mas também se as colunas às quais ele se refere continuam representando os campos corretos, conforme as regras de dependência aprovadas.

Também é importante que o Codex não interprete `getLastRow()` ou `getLastColumn()` isoladamente como erros. Esses métodos dependem do conteúdo e não constituem, por si só, falhas estruturais.

Para arquivos Apps Script, a análise utilizará o código retornado por `projects.getContent`, que permite consultar os arquivos e suas fontes, desde que o projeto esteja autorizado.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers



## 8. Resultado esperado ao finalizar a Sprint 03

Ao concluir, o Sheets Monitor deverá oferecer um motor capaz de produzir diagnósticos semelhantes a estes:

Orçamento 2026

Offline

Falha estrutural confirmada

A coluna "Valor Global", esperada na posição D, foi deslocada para a posição E.

Possível impacto em Apps Script

O método `getRange("D6:D32")` permanece referenciando a coluna D, que não corresponde mais ao campo protegido.

Exemplo de diagnóstico, sem dados reais

A Sprint 03 estará aprovada quando o Codex demonstrar que consegue distinguir uma modificação permitida de uma violação estrutural real, explicar o motivo da classificação e preservar a referência original para futuras verificações.

A próxima etapa será a Sprint 04 — Interface de gerenciamento, responsável por conectar esse motor ao dashboard, cadastro de planilhas, seleção de abas, configuração de regras, histórico e visualização detalhada dos problemas.