# Baselines estruturais

Um baseline é uma versão aprovada do snapshot de uma conta, arquivo e fingerprint de escopo. Somente uma versão está ativa por escopo. O histórico e seus snapshots são imutáveis; nova aprovação altera o ponteiro ativo, sem substituir a versão anterior.

A tela Integridade técnica oferece proposta de escopo, captura e prévia. A proposta inicial considera a primeira aba GRID e até 100 linhas/8colunas; não é aprovação nem descoberta de todas as regras da planilha. O usuário deve revisar cabeçalhos, chave, fórmulas, regiões e scripts. Os índices são zero-based e limites finais exclusivos.

A aprovação exige checkbox explícito e envia snapshot ID, fingerprint integral, geração da sessão e versão anterior esperada. O backend recarrega o conteúdo persistido, valida conta e geração, confere o fingerprint e executa CAS da versão dentro de transação. Reconfigurar/reconectar/trocar conta invalida a confirmação anterior. O IPC usa origem fixa `human:native`; não aceita origem sintética nem snapshot arbitrário do renderer.

Baseline precisa de captura completa, cabeçalhos e âncoras válidos, fórmulas críticas presentes e sem erros, além de dependências críticas verificáveis e válidas. Análise JavaScript sintaticamente completa não basta se a aba/faixa referenciada for inválida. Versão ou escopo desatualizado retorna conflito sem gravação parcial.

Testes usam referências sintéticas aprovadas por especificação; essa via é exclusiva do código de teste. O agente não aprova uma referência real por inferência do login, do acesso à planilha ou de um teste bem-sucedido. Baseline real depende de revisão e aprovação humana na interface.

Retenção de snapshots e eventos é configurada no escopo (padrões 7 e 90 dias; limites 1–365 e 1–3650). A operação é manual nesta sprint. Preserva snapshots de todos os baselines, último snapshot completo e último relatório; estado de incidentes e journal de componentes sobrevivem à limpeza. Versões anteriores continuam consultáveis em páginas, sem cópia automática de valores completos.

Exemplo sintético: aprovar v1 com campos ID/Valor; editar valores gera eventos e mantém Online. Excluir o cabeçalho Valor gera Offline. Corrigir o cabeçalho permite recuperação contra a mesma v1. A correção não cria v2. Uma nova referência só existe após outra aprovação explícita.
