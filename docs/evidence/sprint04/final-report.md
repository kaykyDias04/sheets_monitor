# Sprint 04 — relatório de entrega e validação

SM-018 a SM-021 e adições autorizadas SM-021A/SM-021B implementados. O executável final e o aceite técnico foram aprovados no Gate 3, com revisão adicional do padrão de retenção. Este relatório separa comportamento implementado, testes locais e cobertura Google real; não representa publicação Google para qualquer conta.

## Produto entregue

Seis áreas operacionais: Visão geral, Planilhas, detalhes por arquivo, Atividades, Regras e baselines e Configurações. Dashboard e listagens usam o backend da conta conectada, sem fixtures ou serviço demonstrativo alcançáveis pelo grafo operacional. A configuração técnica anterior permanece separada, em ferramentas avançadas.

Descoberta Drive paginada com busca, ordem, arquivos compartilhados, unidade opcional e indicação de resultado incompleto. Cadastro por escolha explícita ou URL, sem cadastrar automaticamente a lista descoberta. Capacidade transacional de 30 arquivos presentes em todas as contas locais, incluindo pausados; remoção local libera a vaga. Arquivos recém-cadastrados aparecem pendentes, sem Online padrão.

Seleção estável por sheetId, regras estruturadas para abas, cabeçalhos, colunas, ordem, regiões, chaves, fórmulas e scripts críticos/opcionais. O limite de arquivos não é aplicado às abas: até 1.024 abas e 20.000 células por escopo, sujeitos aos limites de coleta. Sugestão lê até 240 células de cabeçalho em até 30 abas, preserva regras válidas e preenche somente um rascunho. Não adivinha fórmulas nem salva/aprova automaticamente.

Captura, revisão, confirmação explícita, referências versionadas e comparação local entre versões. Aprovação fixa conta, geração, revisão do cadastro, fingerprint e versão esperada. Alterações de seleção/regras mantêm configuração pendente até referência correspondente. Análise de scripts da captura pode ser revisada antes da aprovação, sem reutilizar cobertura antiga quando o projeto não foi observado. Nunca se executa código de Apps Script.

Histórico pesquisável com filtros e paginação SQL, diagnóstico por evento e indicação explícita de incerteza. Sem atribuição de autoria sem evidências ou correções automáticas. Remoção exige confirmação e escolha de preservar/excluir atividades; baselines aprovados permanecem imutáveis, com aviso explícito. Preferências persistem, mas agendamento, notificações Windows, bandeja, autostart e instalador continuam fora desta sprint.

## Mudanças e contratos

- Backend: src-tauri/src/management/, leitor google/discovery.rs, guards de aprovação/avaliação em integrity_store/runtime, provisionamento OAuth e comandos nativos em app.rs.
- Interface: OperationalApp, SpreadsheetDetails, ScopeEditor, BaselineFlow, ActivitiesPanel, ScriptCoverage, OperationalSettings, tabelas e componentes compartilhados; serviços/tipos próprios e separação de platform.ts das fixtures antigas.
- Migração 0004: registered_spreadsheets e management_preferences; 24 tabelas de produto. SQL das migrações 0001–0003 preservado.
- 43 comandos Tauri registrados, 20 anteriores preservados. Leitura de regras reutiliza o diagnóstico; não cria comando duplicado equivalente. Detalhes em [contratos](../../api-contracts.md).
- Decisões em [ADR 0009](../../adr/0009-discovery-and-account-management.md). Commits das fases: d6e9977 e f84efae; ajustes de QA e encerramento ficam no commit final da mesma branch codex/sprint-04-interface-management.

## Testes executados

| Verificação | Resultado | Evidência |
|---|---|---|
| Rust MSVC | 188 unitários e 2 integrações aprovados; 1 teste de cofre ignorado por padrão | [rust-test.log](history-default/rust-test.log) |
| React | 42 testes em 8 arquivos aprovados | [frontend-test.log](frontend-test.log) |
| Clippy | Warnings proibidos; aprovado no backend atual | [rust-clippy.log](history-default/rust-clippy.log) |
| Qualidade frontend | ESLint, TypeScript e Prettier aprovados | Logs frontend-lint, frontend-typecheck e frontend-format |
| Builds | Frontend e standalone MSVC aprovados | [frontend-build.log](frontend-build.log), [rust-build.log](rust-build.log) |
| Cenários exigidos | 24 cenários vinculados às suítes | [scenario-matrix.json](scenario-matrix.json) |
| Estrutura | 43 comandos, módulos operacionais sem fixtures | [phase2-structure-scan.json](phase2-structure-scan.json) |

A prova explícita anterior do cofre Windows continua válida porque seu código não mudou; não foi contada como nova execução. Falhas intermediárias permanecem nos logs, incluindo timeouts de dois testes de digitação de URL durante checks concorrentes. As mesmas asserções passaram ao simular colagem de URL, condizente com o fluxo testado. Os logs Rust podem ser UTF-16 devido ao PowerShell; isso não é conteúdo corrompido.

Gate 1 aprovado na segunda revisão. Gate 2 aprovado na terceira, após corrigir sincronização de conta avançada, navegação pendente, limite de abas e abertura segura no navegador. Uma falha visual de contraste no tema automático escuro foi corrigida com tokens e isolamento de CSS legado; os relatórios iniciais foram preservados. A inspeção candidata passou sem violações nas seis áreas. O executável final passou pelo Axe nas seis áreas, sem violações ou verificações incompletas no tema automático escuro. Viewports de 780 e 1280 pixels foram conferidos sem overflow horizontal da página; são medidas do renderer, não um teste de todos os tamanhos de janela. Captura pública com rótulos e inicial da conta omitidos na [captura pública](native-overview-redacted.png). Hash e fechamento no [registro nativo](native-qa.json).

## Google real e persistência

[Leituras reais](live-validation.json) registram sessão protegida recuperada, refresh, duas páginas Drive com um item por página, cadastro pela descoberta e por URL e metadados de 28 abas. Ainda havia páginas disponíveis; não se declara listagem completa da conta nem carga real de 30 arquivos.

No escopo anterior aprovado pelo usuário — uma aba e duas células, sem fórmulas protegidas ou scripts — foram salvas as regras correspondentes, capturada a estrutura e executadas duas verificações Online; a repetição gerou zero eventos novos. A baseline v1 e sua origem humana foram preservadas. Pausa impediu verificação; retomada, comparação da mesma versão e consulta de histórico vazio funcionaram.

Um projeto Apps Script previamente autorizado retornou 23 arquivos e prova de parentId; associação e listagem foram verificadas. Nenhum script foi executado ou editado. Isso não significa descoberta universal de projetos nem homologação de uma baseline com dependências críticas reais.

Dois cadastros, seleção, regras, relatório, referência e intervalo persistiram após reinício. O intervalo de teste foi restaurado de 120 para 60 segundos. Remoção local preservando histórico e restauração do cadastro também foram verificadas. Consultas locais após reinício não exigiram refresh. Esquema 4, 24 tabelas e ausência de erros de chave estrangeira em [native-database.json](native-database.json).

**Estado atual observado:** a seleção foi posteriormente ampliada para 28 abas, enquanto a referência permanece restrita a uma aba. O aplicativo corretamente passou a Configuração pendente e o agente preservou essa seleção. Não se apresenta toda a planilha como protegida; revisar regras e aprovar a referência ampliada é uma ação humana.

## Limites e intervenções humanas

Nova aprovação visual real não foi executada pelo agente: o fluxo foi testado com fixtures e a referência humana anterior foi reutilizada para as leituras. Uma nova baseline de produção depende de revisão/aprovação no aplicativo. Alterações destrutivas Google e recuperação após essas alterações não foram executadas. Histórico não vazio, estados Offline/Indeterminado e concorrência permanecem cobertos localmente por fixtures; a consulta real retornou zero eventos.

O cliente OAuth Desktop da aplicação pode ser provisionado pelo mantenedor no build, sem pedir credenciais Cloud no fluxo diário. A configuração protegida existente foi preservada. Descoberta usa drive.metadata.readonly já previsto, com opt-in e concessão efetiva; desktop requer nova autorização completa quando necessário, sem incremental incompatível. Esse escopo é restrito: distribuição externa ainda depende de projeto autorizado, audiência, políticas e verificação Google aplicáveis. Usuários de teste não comprovam publicação pública. Não há servidor hospedado obrigatório.

Não há remoto Git autorizado; PR apenas em [rascunho local](../../pr-sprint04.md), sem CI hospedado ou publicação remota. Nenhuma identidade, ID de recurso, célula, fórmula, fonte ou credencial real deve integrar os relatórios públicos. Backup privado anterior à migração permanece fora do Git. Encerramento técnico aprovado. Aguardar Prompt 05, sem iniciar a Sprint 05 automaticamente.


## Confirmação final da política padrão

A preferencia de retencao alimenta somente novas protecoes; regras existentes e rascunhos mantem suas politicas. O ultimo ajuste passou em188 unitarios Rust+2 integracoes e42 React, com Clippy/builds aprovados. A revisao de codigo adicional do Gate 3 aceitou esse comportamento; suas condicoes de execucao foram confirmadas pelo principal em [native-history-default.json](native-history-default.json): novo rascunho 365 dias, padrao restaurado90 dias, regras e selecoes salvas preservadas e nenhum rascunho salvo automaticamente.

O artefato atualizado esta identificado em [native-qa.json](native-qa.json) e [final-artifact-after-history.log](final-artifact-after-history.log). O CSS permanece identico ao auditado nas seis rotas; verificacoes nativas adicionais apos o ajuste nao apresentaram violacoes. A [captura de Planilhas](native-spreadsheets-redacted.png) corresponde ao artefato atualizado, com rotulos/inicial omitidos. O estado mais recente observado tem quatro cadastros pendentes e inclui a selecao de 28 abas; as alteracoes de cadastro feitas fora dos testes foram preservadas. Os dois cadastros do teste de reinicio continuam como evidencia historica, sem presumir que a lista da conta permaneceu imutavel.
