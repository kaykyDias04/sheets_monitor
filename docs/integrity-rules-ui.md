# Regras de integridade pela interface

As regras usam o contrato MonitorScope do motor Rust. O frontend edita o contrato e oferece explicações; não calcula equivalência de fórmulas nem decide Online/Offline.

Selecione abas por identidade estável. Cada região define limites, linha de cabeçalho, colunas lógicas e seus nomes esperados. Colunas obrigatórias, identidade e ordem protegidas são opções distintas. Uma coluna-chave identifica registros para movimentos seguros; ausência ou ambiguidade de identidade pode tornar uma comparação inconclusiva.

Fórmulas críticas identificam coluna e linha de referência. Comparação exata exige igualdade; modo estrutural aceita somente equivalências conservadoras reconhecidas pelo motor. Permitir deslocamento de linha não autoriza mudanças arbitrárias. Índices internos são base zero; a interface apresenta linhas e colunas de forma compreensível e converte os valores.

Regiões críticas e dependências críticas afetam a integridade. Regras opcionais geram advertências quando sua verificação falha. A severidade decorre dessas regras e das categorias do motor; não existe mecanismo independente de código executável definido pelo usuário.

Scripts exigem associação autorizada. Intervalos nomeados e referências entre abas são verificados quando reconhecidos pelo motor; dependências dinâmicas aparecem como não verificáveis. Alterar regras invalida a prévia e exige revisão/captura de nova referência para o novo escopo. Referências anteriores não são atualizadas automaticamente.

Limites de captura e análise estão em [motor](integrity-engine.md) e [fórmulas](formula-analysis.md). Não se promete cobertura de células fora das regiões escolhidas.
