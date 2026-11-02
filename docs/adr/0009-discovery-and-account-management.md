# ADR 0009 — Descoberta e operação por conta

Estado: aceito para implementação da Sprint 04. Fontes: docs/sprint04.original.md e docs/spreadsheet-discovery.md. Não altera os 28 IDs originais; SM-021A/SM-021B ficam como adições explícitas.

O aplicativo precisa listar planilhas disponíveis sem exigir links, enquanto a seleção monitorada continua limitada a 30. Usar Drive files.list com filtro MIME e trashed=false, campos explícitos, paginação e suporte compartilhado. Manter drive.metadata.readonly já previsto, com opt-in e concessão efetiva. drive.file não atende a listagem geral. Desktop usa nova autorização completa; não implementar incremental incompatível. Publicação externa depende do projeto autorizado e requisitos Google.

Separar registro operacional e snapshots imutáveis. Pausados contam nas 30 vagas para retomar com segurança. Descoberta não ocupa vaga nem baixa células. Todas as operações locais pertencem à conta e geração autenticadas; mutações usam revisão esperada. Alterações de seleção/regras exigem baseline correspondente antes de reportar integridade atual. Captura não é aprovação. Baselines antigas não são substituídas automaticamente.

A interface principal usa somente dados do backend. Demonstração antiga fica preservada no código/histórico, sem alimentar dashboard operacional. A configuração de escopo é estruturada e usa o contrato Rust; o frontend não implementa comparação de fórmulas. Histórico filtra e pagina no SQLite. Severidade é derivada das regras/categorias do motor, sem lógica executável customizada.

A remoção altera somente dados locais, com escolha de preservar ou excluir atividades. Referências aprovadas são mantidas por sua imutabilidade, inclusive quando atividades são excluídas; essa distinção aparece no fluxo de confirmação. Essa opção não promete apagar toda a base privada local. Ausência de autoria confiável impede atribuições pessoais nos eventos.

O mantenedor provisiona o cliente OAuth Desktop no build; uma configuração protegida local preexistente nunca é sobrescrita na inicialização. O segredo desktop opcional é compatibilidade de cliente público, não mecanismo de confidencialidade. Não versionar credenciais reais. Sem configuração de distribuição autorizada, apresentar pendência de provisionamento ao mantenedor, sem pedir credenciais ao usuário comum.

Preferências de intervalo/notificações são preparatórias. Autostart, execução minimizada, agendador e avisos nativos permanecem indisponíveis até a Sprint 05. Não há servidor hospedado obrigatório.

A seleção de abas tem limite técnico independente de 1.024, com 20.000 células por escopo, sem confundir o teto de 30 arquivos cadastrados. Regressão com 31 abas comprova a distinção.
