# Sprint 01 — gate independente do bridge e shell nativo

Revisor: responsável pelo banco, sem autoria da interface ou do shell Tauri. Tentativa 1/3, revisão estática; builds e navegação visual têm evidências separadas.

## Achados reconciliados

1. O DTO TypeScript `FoundationDashboard` exigia `database_version`, mas o JSON do shell omitia esse campo. O autor do shell adicionou `database.schema_version()` e uma asserção ao teste de serialização.
2. As categorias do seed SQLite divergiam dos filtros de histórico da interface, o que produziria filtros vazios no Windows. O seed foi alinhado ao contrato `content`, `structure`, `operational`; o conteúdo continua marcado como demonstração.

## Limites de escopo conferidos

- Shell expõe somente leitura de dashboard fictício e gravação de preferências locais.
- App data resolve o caminho absoluto `foundation-v1.db`; seed é explícito na inicialização e idempotente pelo banco.
- Não há login, callbacks OAuth, chamada Google, motor de integridade, agendador ou notificação nativa ativos.
- Módulos futuros contêm somente declarações de contratos, sem implementação registrada.
- A interface identifica demonstração, mantém conexão Google e autostart indisponíveis e apresenta o baseline como exemplo sem aprovação.
- O bridge recusa registros sem a marca explícita `is_demo: true`; a prévia web usa memória e identifica que não há SQLite no navegador.
- CSP não admite destinos de rede externos; a janela não recebe permissões de shell ou filesystem no frontend.

Nenhum outro bloqueio material foi identificado nesta revisão estática. Os testes SQLite finais (11 unitários e 2 de integração) passaram em MSVC; não substituem o build nativo e a verificação visual das quatro telas.
