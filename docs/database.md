# SQLite — fundação e integração Google

O módulo `src-tauri/src/database` oferece persistência local e fixtures explícitas. Não executa OAuth, acesso Google, verificações de integridade, agendamento, aprovação de baseline ou notificações Windows.

## Localização e abertura

O shell Tauri resolve seu diretório privado de dados com `app_data_dir()` e fornece um caminho absoluto ao `Database::open`. O módulo cria o diretório necessário, rejeita caminhos relativos e um arquivo de banco que seja link simbólico. `Database::memory()` existe para testes isolados.

Cada conexão habilita chaves estrangeiras, `trusted_schema=OFF`, WAL, sincronização FULL e espera de até cinco segundos por bloqueio SQLite. Não há extensão carregável nem endpoint remoto. Erros públicos usam categorias fixas e não revelam SQL, conteúdo, caminho ou credenciais.

O SQLite não é criptografado pelo aplicativo. As proteções em repouso dependem das permissões da conta Windows e da criptografia de disco. Não há campo de token, client secret ou senha no esquema desta sprint. Bancos e seus arquivos WAL/SHM não devem ser versionados.

## Migrações

`src-tauri/migrations/0001_foundation.sql` define exatamente as onze tabelas de produto previstas na especificação:

| Tabela | Fundação persistida |
| --- | --- |
| `accounts` | Identificação de conta, sem credenciais |
| `spreadsheets` | Arquivo, escopo e estado |
| `monitored_sheets` | Abas selecionadas |
| `baselines` | Versão, JSON e futura aprovação explícita |
| `integrity_rules` | Definições de regra, sem motor de avaliação |
| `script_projects` | Identificação do projeto e futura autorização |
| `script_dependencies` | Referências estruturais ou dinâmicas |
| `monitoring_checks` | Estrutura para registros de verificações |
| `change_events` | Estrutura para histórico e diagnóstico |
| `notifications` | Estrutura para registros de alertas |
| `app_settings` | Preferências locais |

A tabela técnica `_schema_migrations` registra versão, nome, SHA-256 do SQL e data de aplicação. O checksum normaliza apenas CRLF para LF, permitindo a mesma migração em checkout Windows ou CI; outras alterações invalidam a migração aplicada. Migrações já publicadas devem ser preservadas e evoluções devem ganhar novos arquivos/versionamento.

A validação do ledger, execução de todas as migrações pendentes, atualização de `PRAGMA user_version` e gravação do checksum pertencem à mesma transação imediata. Uma falha reverte todas as mudanças dessa tentativa. Versões futuras, ledger inconsistente e checksums divergentes são recusados, sem downgrade ou tentativa automática de reparo.

Chaves estrangeiras, unicidade, JSON válido e restrições de estado são verificadas no banco. Triggers impedem ultrapassar trinta arquivos simultâneos, inclusive na restauração de um arquivo arquivado. A Sprint 01 não expõe cadastro/edição de planilhas reais.

## Demonstração

`Database::open` não cria fixtures automaticamente. O shell pode chamar `seed_demo()` explicitamente na primeira execução. O seed usa uma transação e um marcador persistido; chamadas repetidas não duplicam dados nem sobrescrevem preferências.

Os quatro arquivos fictícios ilustram `online`, `offline`, `checking` e `indeterminate`. Cada registro retornado à interface tem `is_demo: true`; nomes e diagnósticos contêm identificação de demonstração. As URLs usam `example.invalid`, não apontam para arquivos Google. Todos os dados foram definidos localmente, com datas fictícias fixas.

Há exemplos nas onze tabelas para comprovar criação/leitura. A conta fictícia não tem subject Google; o projeto não tem ID externo nem autorização; o baseline não tem aprovação. As notificações têm estado `demo`, sem data de entrega ou operação nativa. Os exemplos de verificações e dependências não são resultado de execução de motores.

## Preferências

- Intervalo padrão: 60 segundos; valores persistíveis entre 60 e 600 segundos.
- Histórico padrão: 90 dias, fixo nesta sprint. Não há job de exclusão nem promessa de retenção aplicada automaticamente.
- Inicialização com Windows: `false`. Tentar salvar `true` é recusado porque a funcionalidade pertence a uma sprint futura.

Salvar intervalo só persiste uma preferência: não inicia polling nem qualquer tarefa de monitoramento.

## Interface pública

`Database` expõe `open`, `memory`, `settings`, `save_settings`, `seed_demo`, `list_demo_spreadsheets`, `demo_history`, `schema_version` e `table_names`. Os dois últimos métodos são diagnósticos da fundação. DTOs `AppSettings`, `DemoSpreadsheet`, `DemoStatus` e `DemoActivity` são serializáveis para o bridge Tauri.

## Verificação reproduzível

Em PowerShell com o ambiente MSVC configurado:

```powershell
. ./scripts/environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
cargo test --manifest-path src-tauri/Cargo.toml --lib --tests --no-default-features database
```

Os testes unitários cobrem leitura das onze tabelas, fixtures atômicas/idempotentes, configurações, chaves estrangeiras (incluindo rejeição de vínculos cruzados entre planilhas), capacidade e migrações. Os testes de integração em `src-tauri/tests/database_persistence.rs` reabrem um banco temporário e verificam persistência, ledger e recusa de versão futura. Resultados executados são registrados em `docs/evidence/`; esta descrição não substitui os logs.

Execução local MSVC em 02/11/2026: **11 testes unitários e 2 testes de integração aprovados**, saída 0. Evidência: `docs/evidence/sprint01-database-tests-final.log`. A primeira tentativa, anterior ao ajuste do target binário com feature desktop obrigatória, falhou ao tentar compilar `main` sem a feature; seu log também foi preservado. Esses testes não declaram validação da interface nativa nem integração Google.

## Evolução Sprint 02

Migração0002 preserva o checksum da 0001, adiciona email/última conexão em accounts e cria integration_config, integration_resources, integration_queries e integration_associations. São 15 tabelas de produto, mais ledger. Configuração pública e metadados consultados são reais e têm is_demo=0; não se misturam às fixtures de monitores. Associações exigem mesma conta e consulta bem-sucedida de metadados Sheets, além da escolha explícita do usuário.

Nenhum token/secret, valor de célula, fórmula ou fonte de script é persistido. O hash de fontes e nomes/IDs/últimos estados são metadados. Logout limpa sessão do cofre, preservando o histórico não secreto no banco; consultas são sempre filtradas por conta. A capacidade 30 refere-se a monitores, não às consultas manuais desta etapa.

## Sprint 03 — engine

Migração 0003 adiciona sete tabelas: integrity_scopes, structural_snapshots, baseline_versions, integrity_reports, integrity_events, integrity_incidents e integrity_observation_components. Total22 tabelas de produto, preservando as quinze anteriores e os checksums 0001/0002. Novas estruturas são separadas dos exemplos da fundação; não duplicam dados reais nas tabelas demonstrativas.

Conta/arquivo/escopo são chaves das operações. Baseline e snapshot são imutáveis; o ponteiro ativo muda em transação CAS. Journal de componentes e revisão observada são separados do último snapshot completo. Retenção não apaga referências aprovadas ou estado de deduplicação. Snapshots mantêm fórmulas protegidas necessárias, cabeçalhos e hashes; valores completos, fontes brutas e tokens continuam fora do banco. Hash de valor não é criptografia nem anonimização forte. O SQLite depende da proteção do perfil/disco Windows.

## Migração 0004 — registro operacional

Duas tabelas novas, registered_spreadsheets e management_preferences, totalizam 24 tabelas de produto. O cadastro usa chave composta conta/arquivo, seleção e escopo JSON validados, revisão monotônica e remoção lógica. Triggers limitam a 30 cadastros presentes em todas as contas locais, contando pausados. Uma atualização do trigger de capacidade de captura deixa de contar cadastros removidos; imagens/baselines existentes permanecem intactas. Não altera SQL das migrações 0001–0003.

Aprovação e commit de avaliação verificam escopo/seleção e revisão atuais na mesma transação. A remoção com exclusão de atividades limpa eventos, relatórios, incidentes e observações, preservando baselines imutáveis; a remoção preservando histórico mantém a consulta de atividades do cadastro arquivado. Operações são sempre segregadas por conta.

## Intervalo operacional atual

O contrato atual aceita somente segundos inteiros entre 60 e 600 nos clientes TypeScript, na validação Rust, nas preferências operacionais e na política do scheduler. A migração aditiva 0006 amplia o CHECK de app_settings preservando os valores existentes e os checksums anteriores. A preferência operacional é por conta; salvar não aprova referências nem inicia o serviço. O serviço ativo replaneja sem sobreposição por arquivo, com até três verificações globais.

## Migração 0007 — versões de referência entre dispositivos

`baseline_versions.sync_version` preserva a versão de origem de uma referência recebida. `version` continua sendo o ordinal local único por escopo; referências com UUIDs distintos e o mesmo ordinal de origem recebem ordinais locais diferentes, sem alterar referências já aprovadas. A sincronização usa a versão de origem nos payloads e preserva os UUIDs.
