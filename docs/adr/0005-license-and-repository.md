# ADR 0005 — licença e governança local

Estado: aceito na fundação.

Não existia LICENSE própria nem remoto configurado para este projeto. Foi feita inspeção de procedência, cabeçalhos e metadados de dependências, sem baixar ou alterar credenciais. A identidade local do projeto é Kayky Dias.

Adotamos MIT somente para código/documentação próprios; terceiros conservam suas licenças. O inventário não equivale a homologação de redistribuição. Avisos e lacunas restantes serão resolvidos antes de SM-028.

Preservamos o commit de ambiente `46ae4fd` e o snapshot do protótipo `ffcaa49`. O trabalho da fundação segue em `codex/sprint-01-foundation`, com commits convencionais vinculados aos IDs oficiais. Sem remoto autorizado, não criar projeto GitHub, não copiar origin de outro projeto e não publicar branches.

Manter o npm lock existente evita introduzir duas resoluções concorrentes durante a fundação. pnpm continua instalado e validado no ambiente; uma migração de gerenciador exigirá decisão separada e evidências, não troca silenciosa.

`.gitignore` impede artefatos locais comuns e padrões de credenciais; evidências sintéticas permanecem versionadas. `.editorconfig` define estilo do código novo, sem reformatar arquivos históricos. Templates de issue/PR servem para futura publicação autorizada.
