# ADR 0006 — stack da fundação e evidência nativa

Estado: aceito na Sprint 01.

O anexo exige Tailwind CSS e shadcn/ui, quatro telas navegáveis, SQLite e prova de execução Windows. A interface anterior e as probes do ambiente não satisfazem esse aceite.

A fundação ativa usa Tailwind via Vite e componentes locais do padrão shadcn sobre primitives Radix, com aliases e separação em páginas, hooks, serviços e tipos. Os dados são sintéticos no desktop e no navegador; apenas no desktop a persistência usa o banco SQLite da fundação. npm permanece o gerenciador reproduzível porque já havia lockfile; pnpm instalado não implica migração.

Os testes da aplicação devem incluir somente código ativo. Arquivo histórico, cópia original da especificação e evidências são excluídos de execução/formatação. Alterações corretivas não reescrevem o conteúdo arquivado.

O crate expõe `sheets_monitor_lib` para evitar colisão de PDB com o binário Windows `sheets-monitor`. O build independente usa `custom-protocol` para incorporar `dist/`; um `cargo build` em modo dev que dependa de Vite não comprova uma aplicação independente. O build script observa mudanças no diretório de assets. Bundle/instalador permanece desativado nesta sprint.

O arquivo SQL tem LF definido por Git e o runner de migração normaliza CRLF antes do checksum, evitando divergência entre Windows e CI. Migração, ledger e versão são atômicos. Não há motor de monitoramento ativo.

A atualização de ferramentas frontend e o override transitivo de desenvolvimento são documentados em [frontend-dependencies.md](../evidence/sprint01/frontend-dependencies.md). Não usar `npm audit fix --force` sem avaliar compatibilidade. Gates incluem instalação limpa, lint, tipos, testes e build; inventário/avisos de terceiros continuam obrigatórios antes de distribuir.

Execução nativa deve ser comprovada pela janela real, navegação e persistência de preferência após reabrir. Mocks de bridge e testes de SQLite isolados fornecem evidências complementares, não substituem essa observação. CI hospedado não é declarado executado sem um run remoto autorizado.
