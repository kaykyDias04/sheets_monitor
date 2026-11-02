# Dependências Apps Script

A análise usa Tree-sitter JavaScript 0.25, sem executar, editar ou publicar código. Apenas fontes SERVER_JS são interpretadas como JavaScript. Manifesto JSON não é execução JavaScript; HTML pode conter scripts inline não analisados e sua cobertura é parcial. A API fornece HEAD, sem promessa de versão publicada.

A proveniência do receptor precisa ser comprovada: SpreadsheetApp.openById literal correspondente ao arquivo monitorado, ou getActiveSpreadsheet em projeto cujo parentId corresponda à planilha. Associação manual não transforma automaticamente um projeto standalone em projeto vinculado. Escopos léxicos, parâmetros, sombreamento, reatribuições, aliases constantes e alterações de métodos são considerados de forma conservadora.

| Padrão | Cobertura inicial |
|---|---|
| getSheetByName com literal | Identifica aba e verifica existência por metadados completos |
| getRange com A1 literal | Converte retângulo e verifica grade/colunas protegidas observáveis |
| getRange com inteiros constantes | Converte índices one-based para zero-based; verifica limites |
| getColumn | Rastreia coluna quando o range/receptor é comprovado |
| getLastRow/getLastColumn | Extensão de conteúdo, sem tratar a chamada isolada como falha |
| getValues/getDisplayValues/getFormulas | Rastreia uso do range conhecido |
| Argumentos ou receptores dinâmicos | Cobertura parcial/não verificável; nunca prova fictícia de validade |

Uma faixa D6:D32 pode continuar executável após mover a coluna lógica para E. O validador compara a identidade de cabeçalhos protegidos aprovada, e não somente se o retângulo cabe na grade. Se a região é parcialmente observada, preserva violações comprovadas na interseção e registra incerteza para o restante. Fora de regiões capturadas, não inventa identidade de coluna.

Categorias: estática verificável, parcialmente verificável, dinâmica não verificável e inválida comprovada. Referência crítica inválida produz Offline; crítica inconclusiva produz Indeterminado. Projeto opcional inacessível gera warning sem impor Indeterminado global se as obrigações foram verificadas. Mudança de hash/arquivo gera evento informativo, independentemente da conclusão semântica.

O parser tem limites 1MiB por arquivo, 4MiB por análise, 128 arquivos, 100 mil nós e profundidade 256, com prazo 250ms por arquivo via progress callback. Erros de sintaxe/limite não são sucesso. Quando a fonte foi obtida mas a análise não completou, hashes permanecem disponíveis para detectar mudança, sem persistir código bruto.

Chaves de incidentes identificam projeto/arquivo/posição/regra, sem incluir o estado encontrado. Mover o mesmo campo incorretamente B→C→D pode atualizar o diagnóstico sem uma recuperação falsa do problema anterior. Comentários e strings não criam chamadas; objetos arbitrários com método getRange não são automaticamente tratados como Sheets.

Limites: não há descoberta universal de projetos, avaliação de código, resolução geral de bibliotecas/imports, argumentos calculados, fluxo arbitrário ou equivalência semântica geral. Não declarar um script inteiro válido apenas porque um padrão simples foi reconhecido. A cobertura consta no snapshot e no relatório.
