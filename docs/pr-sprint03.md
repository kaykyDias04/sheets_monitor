# PR local — Sprint 03

Título: `feat(integrity): add deterministic on-demand structural engine [SM-012..SM-017]`

Antes, o aplicativo autenticava e lia APIs, mas não preservava referência aprovada nem avaliava integridade real. Agora a tela Integridade técnica captura um escopo explícito, apresenta prévia, exige aprovação humana de baseline versionado e valida manualmente estrutura, conteúdo, fórmulas e dependências AST. Uma edição permitida gera evento e pode manter Online; violação crítica comprovada gera Offline; cobertura crítica inconclusiva gera Indeterminado.

O backend separa baseline, última captura completa e journal de observações por componente. Transações e CAS vinculam conta, escopo, geração, fingerprint e versão; baselines são imutáveis. Eventos persistem sem spam e há recuperação/recorrência, inclusive com coleta parcial e retenção. Fontes e valores completos não são persistidos; fórmulas protegidas necessárias ficam no banco privado. Migração0003 preserva as anteriores e acrescenta sete tabelas.

Validação: 146 unitários Rust, 2 de integração e 20 testes UI aprovados; lint, formato, tipos, Clippy e builds React/MSVC aprovados. Medição sintética 30 arquivos×1000 registros: 30 diferenças, máximo 489737 bytes por snapshot, 3458ms na execução registrada. Isso não é benchmark de 30 arquivos reais na API.

Windows nativo confirmou bridge, seis telas, schema 3, quatro fixtures preservadas e acessibilidade da área técnica sem violações. Houve baseline real aprovado pelo usuário e validação Online/repetida sem duplicação, restritos a 1 aba/2 células, sem fórmula/script nesse baseline. Uma captura maior de 800 células/99 linhas, 2 fórmulas e 1 projeto com 23 arquivos/362 dependências permaneceu rascunho não aprovado; a cobertura AST é parcial. Não confundir isso com proteção da planilha inteira ou validação destrutiva real.

Mutações Google não foram executadas; o ciclo destrutivo está comprovado por fixtures. Não há escrita Google, scheduler, notificações, gerenciamento definitivo ou instalador. Gates 1–3 revisados no escopo registrado; não se declara auditoria externa. Sem remoto autorizado, este é um rascunho local; nenhuma PR foi publicada.

Evidências: docs/evidence/sprint03; algoritmos/limites em docs/integrity-engine.md, baselines.md, formula-analysis.md, apps-script-dependencies.md e event-model.md.
