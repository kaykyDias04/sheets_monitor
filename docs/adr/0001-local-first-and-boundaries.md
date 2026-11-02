# ADR 0001 — execução local e limites de confiança

Status: aceito em 01/11/2026.

O repositório começou vazio e herdava um `.git` da pasta pessoal, com remote de outro projeto. Inicializamos um repositório neste diretório, na branch `codex/sheets-monitor-v1`, sem copiar o remote alheio.

Rust é dono de OAuth, HTTP, SQLite, agendamento e notificações. O frontend usa comandos Tauri e recebe DTOs sem tokens. Modo fixture e Google têm bancos separados. Somente IDs válidos são aceitos nas URLs de API, nunca URLs arbitrárias; redirects HTTP são desativados.

Baseline é um snapshot candidato com identificador opaco. A aprovação deve referenciar a prévia específica. Eventos e outbox são persistidos na mesma transação. Entrega de notificações é pelo menos uma vez: falha/reinício pode repetir um toast, mas não deve perder a diferença.

Scripts são recuperados somente para IDs cadastrados explicitamente; análise estática é heurística e não executa fontes. Não há endpoints de escrita de planilhas ou execução de scripts.

Escolhemos polling sequencial com timeout e retry limitados, para conter consumo de quota e evitar sobreposição. A V1 só agenda enquanto o processo está ativo. Todas as diferenças na resposta observada ficam disponíveis; alterações entre polls e campos não retornados são limites documentados.
