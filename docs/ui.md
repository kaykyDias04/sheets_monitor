# Frontend — Sprint 01

A interface ativa corresponde somente à fundação autorizada. O protótipo anterior permanece em `archive/prototype/` e não faz parte da aplicação nem da seleção de testes.

## Estrutura

- `src/pages/`: Dashboard, Planilhas, Histórico e Configurações.
- `src/components/`: composição de tabelas/estados/eventos e componentes shadcn/ui locais.
- `src/hooks/useFoundation.ts`: leitura inicial e salvamento de preferências, com estados de erro e progresso.
- `src/services/foundation.ts`: os únicos comandos nativos são `foundation_get_dashboard` e `foundation_save_settings`.
- `src/types/foundation.ts`: DTOs compartilhados pela UI.
- `src/fixtures.ts`: exemplos sintéticos para a prévia do navegador.

React e TypeScript estrito usam Vite, Tailwind CSS 4 e componentes shadcn/ui baseados em Radix UI (Dialog, Switch e Slot), class-variance-authority, clsx e tailwind-merge. `components.json` registra aliases; `@/` resolve para `src/`. Não há CDN, fontes externas nem scripts remotos. O Tailwind procura classes apenas em `src/`, excluindo o protótipo arquivado.

## Limites da entrega

O selo de demonstração permanece visível também no aplicativo Windows. Os estados Online, Offline, Verificando e Indeterminado ilustram a especificação. Uma alteração comum de valor aparece em exemplo Online e no histórico fictício. Nenhuma avaliação Google, conexão OAuth, aprovação real de baseline, análise real de Apps Script ou notificação de produção é executada.

Dashboard resume fixtures; Planilhas permite busca/filtro e uma prévia didática de baseline; Histórico mostra data, arquivo, categoria e diagnóstico fictícios; Configurações salva intervalo de 60–300 segundos no SQLite pelo backend. Histórico de 90 dias e inicialização com Windows desativada permanecem definidos para esta etapa. Salvar o intervalo não inicia um agendador.

No navegador, os mesmos fluxos usam memória e perdem alterações ao recarregar. No desktop, a persistência é SQLite, através dos comandos locais. O frontend recusa respostas sem a marcação explícita `is_demo`.

## Verificação

Os scripts `npm run typecheck`, `npm run lint`, `npm test`, `npm run build` e `npm run format:check` verificam o código ativo. Vitest inclui somente `src/**/*.test.{ts,tsx}`; testes arquivados não são executados. Os testes cobrem quatro telas, rótulo DEMO, quatro estados, alteração permitida mantendo Online, diálogo baseline sem aprovação, Escape/restauração de foco, preferências via bridge mock, filtros, limites e ausência de comandos Google. Os resultados efetivamente executados estão em `docs/evidence/sprint01/frontend-*.log`.
