# Análise determinística de fórmulas

Fórmulas críticas são configuradas por região, coluna lógica e linha de referência. O baseline mantém o texto necessário à comparação; os demais valores permanecem como hashes. A captura lê `userEnteredValue.formulaValue` e `effectiveValue.errorValue.type`: uma string '#REF!' em célula comum não é erro calculado.

Modos:

| Modo | Regra |
|---|---|
| exact | Protege sintaxe literal e permite apenas deslocamento comprovado pela regra |
| structural | Compara tokens e identidades de alvos no subconjunto suportado; não afirma equivalência geral |

Deslocamento exige regra allow_row_shift e chave única estável. Referências são resolvidas por sheetId, coluna lógica e identidade do registro, preservando flags absolutas/mistas. Intervalos entre abas propagam o qualificador aos dois endpoints. Inserir/excluir linha antes de fórmula pode manter Online quando a âncora e os alvos são demonstravelmente os mesmos.

Excluir um registro com fórmula segue allow_record_deletion. Uma fórmula de registro removido não é equivalente a uma fórmula agregadora obrigatória fixa: a política deve ser explícita. Sem chave, com chave duplicada ou sem observação da célula lógica, não se conclui remoção; a análise é inconclusiva quando obrigatória.

O lexer preserva strings, referências, qualificadores, operadores e literais. Há prova afim muito pequena para um alvo lógico sob operações coercivas +, - e * com inteiros limitados. Exemplo coberto: B2*2 versus 2*B2, com mesmo alvo. B2 versus B2*1 não é equivalência provada: uma string numérica pode se tornar número. Reescritas mais amplas, múltiplos alvos, referências dinâmicas, expansões internas de SUM e construções não suportadas permanecem inconclusivas. IA generativa não decide validade.

Remoção ou substituição por valor só produz violação quando a âncora correspondente foi observada de forma suficiente. Erros REF, NAME, DIVIDE_BY_ZERO, VALUE, NUM, N_A, ERROR e NULL_VALUE em fórmula protegida são violações confirmadas. LOADING é transitório/inconclusivo. Alteração de coeficiente em prova suportada pode confirmar incompatibilidade; mera desigualdade textual estrutural fora da cobertura não produz Offline.

Exemplos locais: =B2*2 deslocada para =B3*2 com chave/alvo preservados →Online; valor24 substituindo fórmula na célula lógica protegida →Offline; referência dinâmica sem prova →Indeterminado; corrigir erro confirmado →evento de recuperação, sem mudar baseline.
