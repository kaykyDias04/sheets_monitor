# Gate 0 — revisão do plano, tentativa 1 de 3

Data: 2026-11-02. Escopo revisado: AGENTS.md, docs/sprint05.original.md, plano `.slim/deepwork/sprint05-continuous-monitoring.md`, interfaces existentes e pesquisa `docs/sprint05-native-research.md`. Nenhuma implementação de produção feita por esta lane antes do GO.

**Parecer: plano aprovado condicionado aos contratos abaixo e à reconciliação pelo principal do baseline em execução.** Este gate aprova a estratégia; não afirma aprovação de código ou execução de testes ainda não concluídos.

1. Um coordenador Rust governa verificações manuais e automáticas, limite global três e exclusão por arquivo; filas e capturas respeitam geração de conta, revisão do cadastro e elegibilidade da baseline atual. Mudanças de conta/cadastro invalidam trabalho antigo antes de persistir/despachar.
2. Retentativas respeitam Retry-After sem clamp prematuro; backoff/jitter e intervalo de agendamento não produzem tempestade de catch-up após suspensão. Falha operacional não transforma integridade em Offline. Retenção e leases de crash preservam eventos/outbox consistentemente.
3. Evento+outbox devem ser atômicos e deduplicados na transação; entrega externa exatamente uma vez não é prometida. Diferenciar erro de preparação, envio submetido, confirmação visual humana e clique. Em notification2.8.0 `show=Ok` não prova sequer aceitação pelo Windows.
4. Permissão desktop do plugin sempre Granted não representa preferência Windows. Opt-in do produto obrigatório; sem envio real durante implementação/testes automáticos. Callback Rust só abre registro validado da conta ativa; toast antigo não concede acesso. Capacidades frontend mínimas.
5. Bandeja é criada antes de habilitar ocultação no X; falha de tray preserva caminho acessível. Single-instance precede inicialização de outros plugins e do serviço. Sair tem cancelamento/limpeza limitados e não depende de React.
6. Autostart é estado Windows por aplicação/usuário e deve ser consultado; não aplicar preferências de conta ao registro automaticamente. Escolher mecanismo que não habilite outros usuários quando elevado. Novas versões oficiais exigem MSRV1.90; refletir manifesto/CI antes de build.
7. Evidência separa mocks, integração Google somente leitura, teste visual humano e limitações do executável não instalado. Cadastros pendentes não autorizam ampliar/aprovar baseline. Sem instalador, não alegar identidade final das notificações.

Gates seguintes: Gate1 algoritmo/persistência e estrutura; Gate2 shell/UI/contas/saída; Gate3 evidências/resultados finais. Cada gate tem revisão inicial e no máximo duas revisões materiais, sem reabrir decisões aceitas inalteradas. Não avançar à Sprint06.
