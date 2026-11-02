# Gate 1 — backend, descoberta e persistência

Revisão especializada das alterações da fase 1. Pesquisa aceita: [descoberta](../../spreadsheet-discovery.md), [Gate 0](gate0.md), ADR 0009. Escopo: autenticação/provisionamento, leitor Drive, registro SQLite, configuração, aprovação, histórico, guards do motor e comandos Tauri. Interface operacional pertence ao Gate 2.

## Tentativa 1 de 3

Achado material **G1-1**: a versão ativa retornada para uma planilha era calculada somente pelo ponteiro ativo de cada escopo. Após mudar regras e aprovar outro escopo, versões antigas podiam continuar marcadas como ativas na listagem operacional. O estado ativo operacional precisa corresponder à configuração atual, preservando os ponteiros internos e o histórico do motor.

A inspeção prévia também identificou e encaminhou correções incorporadas antes do fechamento: erro do cofre não deve abortar a janela; opt-in não deve apagar outras causas de reconexão; descoberta precisa fixar a geração entre opt-in e lease; persistência de relatório exige revisão transacional do cadastro; contagem técnica deve liberar cadastro removido com histórico preservado; entradas IPC precisam de testes reais de desserialização. Essas condições foram verificadas no código, sem chamadas a serviços reais.

## Tentativa 2 de 3

Remediação de G1-1: listagem e consulta por ID passam a combinar o ponteiro do motor com o hash da configuração operacional válida. Regressão cobre dois escopos aprovados e seleção vazia. Na revisão foi apontado um caso adicional da mesma correção: comparação SQL com hash operacional nulo pode retornar NULL e falhar na leitura booleana quando há baseline histórica anterior ao cadastro. O pós-filtro Rust já trata esse estado; a expressão SQL redundante deve ser removida ou normalizada para falso e o caso coberto por teste.

Correção mecânica conferida: expressão SQL redundante removida, pós-filtro Rust mantido. O teste `legacy_baselines_are_listed_inactive_before_operational_rules_are_configured` comprova o caso anterior ao cadastro; a suíte focada atual passou em **18 testes**, incluindo as duas regressões de versões ativas. **Gate 1 aprovado na tentativa 2 de 3**, sem achados materiais pendentes. Nenhuma terceira revisão foi necessária para essa confirmação mecânica.

## Evidências e limites

- Suíte Google focada: 22 testes aprovados, incluindo oito de descoberta e um novo de whitelist — [discovery-tests.log](discovery-tests.log).
- Gerenciamento: 18 testes aprovados, incluindo revisão após pausa/retomada, capacidade após remoção preservando histórico, escopos ativos e hash operacional nulo — [management-tests.log](management-tests.log).
- Preservação das migrações, dependências dos módulos e 22 comandos novos conferidos pelo principal em [structure-scan.json](structure-scan.json).
- O principal executou a suíte completa da fase 1 (176 unitários e duas integrações aprovadas, um teste de cofre ignorado por padrão). As adições posteriores de gerenciamento estão comprovadas pela suíte focada. A compilação e a execução dos dois testes adicionais de pré-condição da descoberta permanecem para a próxima execução completa pelo principal; não foram inferidas por um log anterior.

Revisão de fontes e testes locais; não se declara auditoria externa, descoberta real homologada, aprovação humana de baseline nova ou publicação Google.

Confirmação posterior pelo principal: a suíte completa atual em [phase1-final/rust-test.log](phase1-final/rust-test.log) passou em 180 unitários e duas integrações, com um teste de cofre ignorado por padrão. Inclui as duas pré-condições de descoberta e as 18 regressões de gerenciamento; essa confirmação substitui a pendência de execução mencionada acima.
