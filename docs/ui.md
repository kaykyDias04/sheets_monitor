# Interface desktop e demonstração

A interface React/TypeScript usa apenas comandos Tauri quando executada no aplicativo. O backend é a fonte dos estados, das coletas, das baselines e da fila de notificações. A UI atualiza o dashboard a cada dez segundos; não dispara um segundo agendador de coleta.

## Fluxos

- Visão geral: capacidade de 30 planilhas, cadastro por URL/ID, script IDs explicitamente autorizados, busca, verificação manual e resumo de falhas.
- Cadastro: requer confirmação de consulta somente leitura; nunca aprova baseline implicitamente.
- Baseline: solicita prévia, mostra JSON completo, requer checkbox de revisão e envia o fingerprint opaco da coleta para aprovação. Uma falha não fecha o diálogo nem declara sucesso.
- Histórico: mantém diferenças por caminho, filtra tipo e compara valor anterior/atual em diálogo.
- Apps Script: lista projetos autorizados e mudanças de fonte/manifest; explica análise estática sem execução.
- Notificações: mostra uma entrada por diferença, tentativas e estado informado pelo backend. Reprocessar não significa entrega bem-sucedida por si só.
- Configurações: client ID desktop, conexão/desconexão Google, modo de dados e intervalo. A troca de modo recarrega o dashboard e usa os bancos separados pelo backend.
- Exportação: backend fornece o caminho de relatório local; o frontend não solicita caminhos arbitrários.
- Remoção: requer digitar o nome, arquiva acompanhamento no backend e não altera arquivos Google.

## Demonstração no navegador

`npm run dev` sem Tauri habilita somente um adaptador em memória, com aviso persistente. Os nomes, snapshots e alterações são fictícios, a sessão se perde ao recarregar e a conta permanece desconectada. Google, exportação para disco e notificações Windows retornam indisponibilidade explícita. Os testes do adaptador demonstrativo não constituem evidência de integração Google ou do backend Rust.

## Acessibilidade e apresentação

Layout responsivo, navegação com nome acessível mesmo quando reduzida a ícones, rótulos nos formulários, foco visível, diálogos com captura/restauração de foco e fechamento por Escape. Erros e progresso são anunciados por regiões de status; botões bloqueiam ações repetidas durante operações. Preferência por movimento reduzido desativa animações. Não há CDN, fontes externas ou scripts remotos.

## Verificações locais

`npm test` cobre aprovação explícita da baseline, cadastro e deduplicação de scripts, comparação de valores, remoção confirmada, ausência de falsos sucessos Google/notificações, rejeição de fingerprints inválidos, capacidade de 30 e evento/fila por diferença no adaptador de demonstração. `npm run build` faz checagem TypeScript estrita e bundle de produção. Resultados efetivamente executados em `docs/evidence/frontend-tests.log` e `frontend-build.log`.
