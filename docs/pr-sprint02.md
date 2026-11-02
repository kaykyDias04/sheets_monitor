# PR local preparado — Sprint 02

Título: `feat(google): add desktop OAuth and manual Workspace readers [SM-007..SM-011]`

O aplicativo passa a oferecer a tela Integração Google, com OAuth Desktop PKCE pelo navegador, cofre Windows, identidade/renovação/logout e revogação explícita. Leituras manuais consultam Sheets (abas, metadados, valores e fórmulas), Drive metadata opt-in e projetos/fontes Apps Script por ID, sem modificar recursos Google. SQLite evolui por migração0002 para metadados separados por conta; conteúdo e tokens não são persistidos no banco.

Validação local: 13 testes React, 43 unitários Rust e 2 de integração aprovados; teste adicional de cofre Windows aprovado com dados sintéticos; lint, formato, tipos, Clippy sem warnings, builds React/MSVC e gates de repositório aprovados. Renderer nativo com cinco telas e bridge, migração2 e acessibilidade Google sem violações. Evidências e limites em docs/evidence/sprint02 e docs/progress.md.

Homologação real permanece pendente de cliente Desktop, consentimento e recursos de teste. Não marcar SM-007 a SM-011 como concluídos por mocks. Revisão realizada pelo agente principal; lanes anteriores ficaram indisponíveis, sem gate independente concluído. Não há motor de integridade, baseline aprovada, polling, notificações ou instalador. Origem GitHub não configurada/autorizada; este texto é rascunho local, sem PR publicado.
