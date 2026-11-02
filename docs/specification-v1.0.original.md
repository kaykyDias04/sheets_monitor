# Sheets Monitor — Escopo técnico fechado e backlog inicial

DOCUMENTO DE ESPECIFICAÇÃO · VERSÃO 1.0 · WINDOWS 11

O Sheets Monitor será um aplicativo desktop para Windows 11, desenvolvido integralmente por agentes de IA no Codex, com o objetivo de monitorar até 30 planilhas Google Sheets, detectar alterações estruturais e de conteúdo, analisar dependências de Apps Script e notificar o usuário sobre todas as modificações identificadas.

A arquitetura será local, sem servidor próprio e sem webapp. O Codex terá autonomia operacional para instalar dependências, produzir código, executar testes, corrigir erros e preparar o instalador. A intervenção humana será limitada às autorizações indispensáveis e à homologação final.

## 1. Escopo aprovado

Sistema

Windows 11

Capacidade

Até 30 planilhas

Apps Script

Desde a V1.0

Notificações

Todas as alterações

Execução

Segundo plano

Armazenamento

SQLite local

### Requisitos funcionais obrigatórios

| Código | Funcionalidade  | Comportamento                                          |
| ------ | --------------- | ------------------------------------------------------ |
| RF-01  | Login Google    | Autenticação OAuth 2.0 com PKCE                        |
| RF-02  | Cadastro        | Adicionar planilhas por link                           |
| RF-03  | Gerenciamento   | Editar, remover e pausar monitoramento                 |
| RF-04  | Seleção de abas | Monitorar arquivo inteiro ou abas específicas          |
| RF-05  | Baseline        | Registrar a estrutura inicial aprovada                 |
| RF-06  | Estrutura       | Identificar colunas alteradas, excluídas ou deslocadas |
| RF-07  | Fórmulas        | Detectar alterações e erros em fórmulas protegidas     |
| RF-08  | Apps Script     | Monitorar código acessível e dependências estruturais  |
| RF-09  | Conteúdo        | Detectar diferenças de linhas e valores                |
| RF-10  | Notificações    | Alertar sobre todas as diferenças detectadas           |
| RF-11  | Histórico       | Registrar eventos com data, arquivo e diagnóstico      |
| RF-12  | Dashboard       | Exibir integridade de todas as planilhas               |
| RF-13  | Segundo plano   | Continuar verificando enquanto minimizado              |
| RF-14  | Recuperação     | Atualizar o status após correção de problemas          |

### Definição dos estados

Online

A planilha foi verificada e está estruturalmente íntegra.

Offline

Há falha confirmada em estrutura, fórmula protegida ou dependência crítica.

Verificando

O aplicativo está executando a avaliação.

Indeterminado

Não foi possível confirmar a integridade por indisponibilidade, permissão ou falha técnica.

Uma alteração permitida, como editar um valor, deverá gerar notificação, mas não deverá colocar a planilha em Offline.

## 2. Arquitetura técnica definitiva

| Camada             | Tecnologia adotada                 |
| ------------------ | ---------------------------------- |
| Aplicativo Windows | Tauri 2                            |
| Interface          | React + TypeScript + Vite          |
| Componentes        | Tailwind CSS + shadcn/ui           |
| Backend local      | Rust                               |
| Banco de dados     | SQLite                             |
| Autenticação       | Google OAuth 2.0 com PKCE          |
| Integração         | Google Sheets API v4               |
| Metadados          | Google Drive API v3                |
| Scripts            | Google Apps Script API             |
| Credenciais        | Armazenamento seguro do Windows    |
| Notificações       | Notificações nativas do Windows 11 |
| Versionamento      | Git + GitHub                       |
| Testes             | Vitest, testes Rust e integração   |
| Instalador         | Windows x64, `.exe` ou `.msi`      |

Tauri exige ferramentas de compilação C++ e WebView2 no Windows; o Codex deverá verificar sua presença e automatizar a configuração quando possível.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://tauri.app\&sz=32)

Tauri+1



### Parâmetros operacionais

| Parâmetro                            | Decisão                         |
| ------------------------------------ | ------------------------------- |
| Intervalo inicial                    | 60 segundos                     |
| Intervalo configurável               | 60–300 segundos                 |
| Verificações paralelas               | Até 3                           |
| Histórico padrão                     | 90 dias                         |
| Baseline                             | Versionado, aprovação explícita |
| Notificações repetidas               | Consolidação sem perder eventos |
| Execução minimizada                  | Sim                             |
| Inicializar com Windows              | Opcional                        |
| Correção automática das planilhas    | Não                             |
| Modificação de Apps Script           | Não                             |
| Servidor externo próprio             | Não                             |
| Múltiplos computadores sincronizados | Fora da V1.0                    |

O limite de 30 refere-se a arquivos cadastrados simultaneamente. O aplicativo deverá informar quando uma planilha for muito grande para cumprir a frequência configurada dentro das cotas disponíveis.

## 3. Regras de integridade e Apps Script

O sistema deverá comparar a estrutura atual com uma referência aprovada (baseline).

| Alteração                     | Notificar               | Status esperado            |
| ----------------------------- | ----------------------- | -------------------------- |
| Inserir linha comum           | Sim                     | Online                     |
| Excluir linha comum           | Sim                     | Online                     |
| Editar valor                  | Sim                     | Online                     |
| Reordenar coluna protegida    | Sim                     | Offline                    |
| Excluir coluna protegida      | Sim                     | Offline                    |
| Alterar fórmula crítica       | Sim                     | Offline, se violar a regra |
| Quebrar fórmula crítica       | Sim                     | Offline                    |
| Excluir aba monitorada        | Sim                     | Offline                    |
| Alterar código Apps Script    | Sim                     | Conforme impacto           |
| Invalidar dependência crítica | Sim                     | Offline                    |
| Perder conexão                | Sim, evento operacional | Indeterminado              |

### Tratamento de Apps Script

O monitoramento estará incluído desde a primeira versão, com escopo tecnicamente delimitado:

1. Ler o código dos projetos Apps Script autorizados, utilizando os respectivos IDs.
2. Comparar o conteúdo atual com a versão de referência.
3. Identificar referências estáticas como `getSheetByName()`, `getRange()` e índices de colunas.
4. Verificar se as referências estruturais continuam válidas.
5. Sinalizar dependências dinâmicas que não possam ser verificadas com segurança.

A Apps Script API permite ler arquivos de código e metadados de projetos autorizados por meio de `projects.getContent`, com o escopo `script.projects.readonly`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://developers.google.com\&sz=32)

Google for Developers+1



Limitação assumida: não será possível garantir a validação automática de todo comportamento de scripts dinâmicos nem detectar toda edição intermediária de planilhas apenas com consultas periódicas. O aplicativo registrará todas as diferenças efetivamente observadas, sem prometer auditoria instantânea irrestrita.

# 4. Backlog inicial de desenvolvimento

O desenvolvimento será organizado em seis sprints, com dependências explícitas e critérios de aceite.

Roadmap de implementação

6 sprints

Sprint 01 · Fundação do aplicativo

3–5 dias

Sprint 02 · Autenticação e Google APIs

4–7 dias

Sprint 03 · Motor de integridade

6–10 dias

Sprint 04 · Interface e gerenciamento

4–6 dias

Sprint 05 · Monitoramento e alertas

4–6 dias

Sprint 06 · Testes e distribuição

4–7 dias

As barras representam esforço relativo ilustrativo, não percentual de conclusão.

## Sprint 01 — Fundação e infraestrutura

| ID     | Tarefa                                | Critério de aceite       |
| ------ | ------------------------------------- | ------------------------ |
| SM-001 | Verificar e preparar ambiente Windows | Dependências instaladas  |
| SM-002 | Criar repositório e projeto Tauri     | Aplicativo inicia        |
| SM-003 | Configurar React, TypeScript e UI     | Interface navegável      |
| SM-004 | Estruturar backend Rust               | Módulos organizados      |
| SM-005 | Criar SQLite e migrações              | Persistência testada     |
| SM-006 | Configurar testes e documentação      | Build e testes aprovados |

## Sprint 02 — Autenticação e Google APIs

| ID     | Tarefa                                     | Critério de aceite      |
| ------ | ------------------------------------------ | ----------------------- |
| SM-007 | Implementar OAuth com PKCE                 | Login Google funcional  |
| SM-008 | Implementar armazenamento seguro de tokens | Sessão recuperável      |
| SM-009 | Integrar Sheets API                        | Ler planilha autorizada |
| SM-010 | Integrar Drive API                         | Ler metadados           |
| SM-011 | Integrar Apps Script API                   | Ler projeto autorizado  |

## Sprint 03 — Motor de integridade

| ID     | Tarefa                                | Critério de aceite              |
| ------ | ------------------------------------- | ------------------------------- |
| SM-012 | Implementar baseline versionado       | Referência persistida           |
| SM-013 | Detectar alterações de abas e colunas | Diferenças identificadas        |
| SM-014 | Comparar fórmulas protegidas          | Falhas detectadas               |
| SM-015 | Analisar dependências Apps Script     | Referências verificadas         |
| SM-016 | Classificar integridade               | Estados corretos                |
| SM-017 | Registrar mudanças de conteúdo        | Eventos sem duplicação indevida |

## Sprint 04 — Interface desktop

| ID     | Tarefa                        | Critério de aceite             |
| ------ | ----------------------------- | ------------------------------ |
| SM-018 | Criar dashboard               | Indicadores funcionais         |
| SM-019 | Implementar cadastro por link | Adicionar/remover arquivos     |
| SM-020 | Criar seleção de abas         | Escopo individual configurável |
| SM-021 | Criar histórico               | Eventos pesquisáveis           |

## Sprint 05 — Monitoramento contínuo

| ID     | Tarefa                            | Critério de aceite             |
| ------ | --------------------------------- | ------------------------------ |
| SM-022 | Criar agendador                   | Consultas automáticas          |
| SM-023 | Criar notificações nativas        | Alertas no Windows             |
| SM-024 | Implementar bandeja do sistema    | Monitoramento minimizado       |
| SM-025 | Implementar recuperação de falhas | Tratamento de internet e cotas |

## Sprint 06 — Homologação e distribuição

| ID     | Tarefa                          | Critério de aceite          |
| ------ | ------------------------------- | --------------------------- |
| SM-026 | Testes integrados               | Cenários críticos aprovados |
| SM-027 | Teste de carga com 30 planilhas | Estabilidade comprovada     |
| SM-028 | Gerar instalador                | Instalação Windows validada |

Estimativa inicial: 25–41 dias úteis para um desenvolvimento incremental assistido por IA. É uma previsão de planejamento, não uma garantia de prazo.

# 5. Primeira fase detalhada — Sprint 01

Prioridade imediata

A primeira fase terá como meta criar a fundação executável do aplicativo sem implementar antecipadamente o monitoramento Google.

O Codex deverá ser responsável por todo o trabalho técnico possível.

## 5.1. Sequência operacional

01

Auditoria do ambiente

Verificar Windows, Git, Node.js, Rust, Cargo, C++ Build Tools e WebView2.

Entrega: Relatório de dependências

02

Instalação automatizada

Instalar dependências ausentes por meios oficiais quando o ambiente permitir.

Entrega: Ambiente preparado

03

Inicialização do repositório

Criar estrutura Git, .gitignore, documentação inicial e branch de trabalho.

Entrega: Repositório versionado

04

Criação do Tauri

Configurar Tauri 2, React, TypeScript, Vite e Rust.

Entrega: Janela desktop executável

05

Construção visual

Implementar dashboard simulado, navegação lateral e telas básicas.

Entrega: Interface navegável

06

Banco local

Configurar SQLite, tabelas, migrações e testes de persistência.

Entrega: Banco funcional

07

Testes e documentação

Executar compilação, lint, testes e registrar resultados.

Entrega: Sprint homologável

## 5.2. Estrutura inicial do repositório

```
sheets-monitor/
├── src/
│   ├── components/
│   ├── pages/
│   ├── hooks/
│   ├── services/
│   └── types/
├── src-tauri/
│   ├── src/
│   │   ├── auth/
│   │   ├── google/
│   │   ├── integrity/
│   │   ├── monitoring/
│   │   ├── notifications/
│   │   └── database/
│   └── migrations/
├── tests/
├── docs/
├── .github/workflows/
├── AGENTS.md
├── README.md
└── package.json
```

Os módulos deverão existir inicialmente como estruturas ou contratos, sem implementações fictícias apresentadas como funcionalidades reais.

## 5.3. Estrutura inicial do banco de dados

| Tabela                | Finalidade                 |
| --------------------- | -------------------------- |
| `accounts`            | Contas Google autorizadas  |
| `spreadsheets`        | Cadastro dos arquivos      |
| `monitored_sheets`    | Abas selecionadas          |
| `baselines`           | Referências aprovadas      |
| `integrity_rules`     | Regras de integridade      |
| `script_projects`     | Projetos Apps Script       |
| `script_dependencies` | Dependências identificadas |
| `monitoring_checks`   | Histórico de verificações  |
| `change_events`       | Alterações detectadas      |
| `notifications`       | Alertas emitidos           |
| `app_settings`        | Configurações locais       |

O Codex deverá configurar migrações versionadas e testes que comprovem a criação e a leitura dessas tabelas.

# 6. Intervenções humanas: o mínimo indispensável

O objetivo é que o usuário participe apenas quando houver uma exigência que não possa ser resolvida autonomamente.

| Atividade                  | Responsável principal              | Ação humana                                    |
| -------------------------- | ---------------------------------- | ---------------------------------------------- |
| Instalar ferramentas       | Codex                              | Aprovar elevação de privilégios, se necessária |
| Criar projeto e arquivos   | Codex                              | Nenhuma                                        |
| Escrever e testar código   | Codex                              | Nenhuma                                        |
| Configurar Git             | Codex                              | Login, caso a credencial não esteja disponível |
| Criar projeto Google Cloud | Codex, se houver acesso autorizado | Login e autorizações necessárias               |
| Configurar OAuth           | Codex                              | Consentimento e eventual configuração restrita |
| Habilitar APIs             | Codex, quando autorizado           | Aprovação de permissões                        |
| Criar planilhas fictícias  | Codex                              | Nenhuma, com credenciais adequadas             |
| Cadastrar planilhas reais  | Aplicativo                         | Inserir links e conceder acesso                |
| Aprovar baseline           | Usuário                            | Confirmar a estrutura correta                  |
| Executar testes            | Codex                              | Nenhuma nos testes automatizados               |
| Homologar funcionamento    | Usuário                            | Avaliar a entrega                              |
| Criar instalador           | Codex                              | Assinatura, se exigida para distribuição       |

A configuração de OAuth e o acesso a projetos Apps Script podem exigir consentimento e permissões específicas, mesmo que a criação da infraestrutura seja automatizada.

O Codex não deve solicitar senhas, tokens privados ou códigos de autenticação para incluí-los diretamente no código-fonte.

## 6.1. Checklist de preparação humana

Checklist de responsabilidades

0/8

Disponibilizar um computador Windows 11 para desenvolvimento

Disponibilizar acesso ao Codex com terminal e arquivos do projeto

Autorizar instalações administrativas indispensáveis

Autenticar GitHub, caso seja necessário publicar o repositório

Disponibilizar conta Google para testes de integração

Autorizar os escopos OAuth necessários

Confirmar a estrutura correta das planilhas reais

Validar o aplicativo final

&#x20;Copiar checklist

# 7. Comando inicial para o Codex — Sprint 01

O prompt abaixo deverá ser executado após o Prompt 00 de contrato mestre, que estabelece as regras gerais de autonomia. Assim, a IA terá liberdade para executar toda a primeira fase, mas não avançará para a Sprint 02 sem uma nova instrução.

# PROMPT 01 — Implementação da fundação do Sheets Monitor

Você é o agente de engenharia responsável pela Sprint 01 do projeto Sheets Monitor.

## Contexto

O Sheets Monitor é um aplicativo desktop para Windows 11, com capacidade de monitorar até 30 planilhas Google Sheets, incluindo alterações estruturais, fórmulas e dependências de Apps Script.

Tecnologias obrigatórias:

- Tauri 2
- React e TypeScript
- Rust
- SQLite
- Tailwind CSS e shadcn/ui

## Objetivo

Executar integralmente a primeira fase de desenvolvimento, criando a fundação técnica do aplicativo.

Não implementar ainda a autenticação Google nem o monitoramento real das planilhas.

## Autonomia de execução

Realize autonomamente todas as tarefas técnicas possíveis.

Você está autorizado a:

1. Inspecionar o ambiente.
2. Instalar dependências de desenvolvimento necessárias, respeitando as permissões disponíveis.
3. Criar arquivos e diretórios.
4. Inicializar o projeto.
5. Escrever e modificar código.
6. Executar comandos e testes.
7. Corrigir erros de implementação.
8. Gerar documentação.
9. Criar commits em uma branch de trabalho.

Não solicite confirmação para decisões técnicas rotineiras e reversíveis dentro desta sprint.

Peça intervenção humana somente quando houver autenticação, elevação de privilégios, falta de credenciais ou outra restrição que não possa ser resolvida automaticamente.

## Tarefas

### Etapa 1 — Diagnóstico

Verifique todas as dependências necessárias para desenvolver o aplicativo no Windows 11.

Instale as dependências ausentes quando permitido.

### Etapa 2 — Fundação

Crie o projeto Tauri 2 com React, TypeScript, Vite e Rust.

Organize o código em módulos com responsabilidades separadas.

### Etapa 3 — Interface

Implemente um layout inicial com:

- Dashboard
- Planilhas cadastradas
- Histórico de atividades
- Configurações

Utilize dados simulados, sempre identificados como demonstração.

### Etapa 4 — Persistência

Configure SQLite e crie as migrações iniciais para as entidades previstas.

Implemente testes de persistência.

### Etapa 5 — Qualidade

Configure:

- Testes unitários
- Verificação de tipos
- Lint
- Formatação
- Build automatizado
- Documentação
- CI compatível com Windows

### Etapa 6 — Verificação

Execute os testes e corrija automaticamente os erros encontrados.

Registre os resultados reais da compilação e execução.

## Critérios de conclusão

A sprint somente poderá ser considerada concluída quando:

1. O aplicativo abrir corretamente no Windows 11.
2. A interface permitir navegar entre as telas.
3. O SQLite estiver funcional.
4. Os testes automatizados pertinentes passarem.
5. O projeto compilar.
6. A documentação estiver atualizada.
7. Não houver credenciais expostas no repositório.

## Relatório final

Apresente:

- Arquivos criados e modificados.
- Funcionalidades concluídas.
- Testes executados e resultados.
- Problemas encontrados.
- Pendências humanas.
- Instruções para executar o aplicativo.
- Estado de preparação para a Sprint 02.

Não declare sucesso sem evidência de execução.

## Restrição final

Execute somente a Sprint 01.

Não avance para as tarefas da Sprint 02, mesmo que a primeira fase seja concluída sem erros.

Ao finalizar, encerre a execução e aguarde o próximo prompt.

## 8. Critério de encerramento da primeira fase

A Sprint 01 será aprovada quando o Codex apresentar o aplicativo em execução no Windows 11, navegação funcional, banco SQLite operacional, testes aprovados e documentação reproduzível.

Caso o ambiente não permita a compilação nativa, o Codex deverá informar exatamente quais testes conseguiu executar e quais dependem de um computador Windows; não poderá substituir a homologação por uma declaração de sucesso.

A estratégia recomendada é manter autonomia máxima dentro de cada sprint e controle de avanço entre sprints. Isso reduz substancialmente a participação humana sem permitir que o Codex implemente toda a aplicação de uma vez, antes de validar suas fundações.