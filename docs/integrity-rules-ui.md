# Regras de integridade pela interface

As regras usam o contrato MonitorScope do motor Rust. O frontend edita o contrato e oferece explicações; não calcula equivalência de fórmulas nem decide Online/Offline.

Selecione abas por identidade estável. Cada região define limites, linha de cabeçalho, colunas lógicas e seus nomes esperados. Colunas obrigatórias, identidade e ordem protegidas são opções distintas. Uma coluna-chave identifica registros para movimentos seguros; ausência ou ambiguidade de identidade pode tornar uma comparação inconclusiva.

Fórmulas críticas identificam coluna e linha de referência. Comparação exata exige igualdade; modo estrutural aceita somente equivalências conservadoras reconhecidas pelo motor. Permitir deslocamento de linha não autoriza mudanças arbitrárias. Índices internos são base zero; a interface apresenta linhas e colunas de forma compreensível e converte os valores.

Regiões críticas e dependências críticas afetam a integridade. Regras opcionais geram advertências quando sua verificação falha. A severidade decorre dessas regras e das categorias do motor; não existe mecanismo independente de código executável definido pelo usuário.

Scripts exigem associação autorizada. Intervalos nomeados e referências entre abas são verificados quando reconhecidos pelo motor; dependências dinâmicas aparecem como não verificáveis. Alterar regras invalida a prévia e exige revisão/captura de nova referência para o novo escopo. Referências anteriores não são atualizadas automaticamente.

Limites de captura e análise estão em [motor](integrity-engine.md) e [fórmulas](formula-analysis.md). Não se promete cobertura de células fora das regiões escolhidas.

A ação de sugerir regras lê somente cabeçalhos das abas selecionadas, com limite de oito colunas por aba e 240 células ao todo. Regras existentes válidas são preservadas; novas regiões propostas têm até 50 linhas. Não há inferência automática de fórmulas críticas ou projetos de script. Cabeçalhos vazios/ambíguos exigem revisão manual. A sugestão preenche somente o rascunho: salvar regras e aprovar uma referência continuam ações explícitas.

O limite da sugestão é de três chamadas lógicas ao leitor, incluindo uma repetição após 401. Cada leitor mantém os retries limitados do transporte existente; por isso pode haver até nove tentativas HTTP, dentro dos prazos limitados. O refresh OAuth tem seu próprio limite. Isso não deve ser descrito como somente três requisições físicas.

O limite de 30 refere-se a arquivos cadastrados, não ao número de abas. A seleção e o contrato do motor aceitam até 1.024 abas, mantendo o limite total de 20.000 células por escopo e os limites de payload/tempo. A sugestão automática de cabeçalhos continua limitada a 30 abas por ação; seleções maiores podem ser configuradas manualmente no editor. O coletor continua dividindo leituras e informando cobertura incompleta quando seus limites forem atingidos.

A política padrão de histórico das Configurações alimenta o rascunho de novas proteções e a sugestão do backend quando ainda não existe escopo salvo. Alterá-la preserva as políticas individuais já configuradas e o rascunho em edição. O prazo de cada proteção permanece revisável no editor, antes de aprovar a referência.
