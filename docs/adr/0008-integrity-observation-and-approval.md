# ADR 0008 — Integridade determinística, aprovação e observações parciais

Estado: aceito na Sprint 03, após os Gates 1 e 2. Escopo: SM-012 a SM-017, operações sob demanda, sem scheduler ou escrita Google.

Snapshots schema 1 têm fingerprint do escopo integral: abas, regiões, identidade lógica de colunas, cabeçalhos, chaves, fórmulas, criticidade, scripts e retenção. SheetId identifica abas. A região define uma janela de captura, não uma quantidade imutável de linhas de dados. Reduzir gridRowCount com exclusão permitida não desloca essa janela; geometria crítica de named ranges é comparada separadamente.

GridData é validado antes de assumir ausência de dados. Erros efetivos são distintos de texto literal '#REF!'. Valores e linhas viram hashes; somente fórmulas protegidas, cabeçalhos e metadados necessários são mantidos. Fontes Apps Script são transitórias; persistem hashes e análise AST. Limites de tamanho, células, concorrência, nós e tempo são explícitos. Não há promessa de leitura atômica entre APIs.

Baseline é imutável e versionado por conta, arquivo e escopo. Aprovação vincula conteúdo, fingerprint, geração da sessão e versão anterior esperada, revalidando tudo na transação. O IPC exige confirmação humana e não oferece origem sintética. Uma geração nova invalida a prévia anterior, mesmo que a mesma conta tenha sido reconectada.

Integridade usa a referência aprovada; conteúdo usa a última observação completa de cada componente. Um journal limitado a metadados, regiões completas e scripts observados preserva diferenças e reversões parciais. O último snapshot global completo continua separado. CAS compara baseline ativo, snapshot global anterior e revisão do journal. Journal e incidentes sobrevivem à retenção; referências e seus snapshots são preservados.

Violações críticas comprovadas têm precedência sobre inconclusões sem ocultá-las. Falha técnica não significa estrutura removida nem resolve um incidente anterior. Uma falha opcional não vira automaticamente incerteza crítica global. Sem chave ou identidade suficiente, registra-se mudança sem inventar inserção, exclusão ou equivalência.

Fórmulas preservam flags absolutas e resolvem o subconjunto coberto por sheetId, coluna lógica e chave. Intervalos entre abas herdam qualificador nos dois endpoints. A prova afim pequena exige operações coercivas nos dois lados; referência isolada não é considerada equivalente a multiplicar por 1 sem prova de tipo. Construções fora da cobertura permanecem inconclusivas.

JavaScript usa Tree-sitter 0.25.10 e grammar 0.25.0 com proveniência, escopos e sombreamento. Hash alterado gera evento; só dependência crítica comprovadamente inválida gera Offline. Análise sintática completa não autoriza baseline com dependência estrutural inválida. HTML e referências dinâmicas têm cobertura parcial explícita.

Revisões: Gate1 corrigiu seis achados e um risco adicional de coerção, com regressões; Gate2 confirmou a vinculação da prévia à geração e a criticidade por componente. O Gate3 confronta build nativo, evidências reais sanitizadas e limites declarados. Detecção usa o horário da análise; captura mantém seus horários separados, para uma análise tardia não nascer expirada na retenção.

Fontes: [Tree-sitter](https://docs.rs/tree-sitter/0.25.10/tree_sitter/struct.Parser.html), [grammar JavaScript](https://docs.rs/tree-sitter-javascript/0.25.0/tree_sitter_javascript/), [Sheets.get](https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets/get), [GridData](https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets/sheets), [ExtendedValue](https://developers.google.com/workspace/sheets/api/reference/rest/v4/spreadsheets/other#ExtendedValue).
