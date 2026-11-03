# Gate 1 — backend e persistência, tentativa 1 de 3

Revisão de fonte em 2026-11-02 antes de integração OS. Base: contratos Gate0, monitoring/{service,work,policy,models}, database/monitoring_store, migration0005, persistência de integridade guardada, transporte Google e notifications/service. Não executar Google/cofre real.

Parecer inicial: **aguardar remediação e evidências focadas** dos itens abaixo. A lane SQLite apresentou 71 testes database aprovados (docs/evidence/sprint05/database-tests.md); a lane scheduler executa cobertura adicional. Este registro não afirma resultado de testes ainda em execução.

## Achados materiais

1. **P1 — preservar Retry-After ao replanejar.** Replanejamento por restart, pausa/retomada, intervalo ou gap substituía next_run_ms por stagger a partir de agora, antecipando embargo persistido de 429. Manual também não deve ignorar esse embargo. Remediação aceita: preservar limite futuro de falha ou campo próprio, com teste de relógio sintético de Retry-After longo seguido por restart/retomada/intervalo e pedido manual.
2. **P1 — finalização operacional precisa da geração de autenticação.** finish_monitoring_check era chamado fora de auth.with_generation, permitindo gravar falha de sessão encerrada antes do tick de cancelamento. Remediação aceita: guard curto de geração na finalização; liberação CAS de lease antigo pode ocorrer sem criar eventos da sessão antiga. Exigir teste de troca/logout durante falha.
3. **P2 — falha de criação da thread não pode bloquear arquivo permanentemente.** Após claim e inserção em active, spawnErr deixava active ocupado sem worker. Remover entrada e liberar/cancelar lease pertencente à tentativa, sem fabricar indisponibilidade; agendamento deve continuar recuperável.
4. **P2 — verificar o dispatcher com sink falso.** Testes de banco não exercitam authguard + submit + ack. Precisam comprovar preferência desligada sem chamada, geração/conta obsoleta sem chamada, submissão terminal sem reenvio em restart e erro de preparação retentável. Shutdown deve verificar stopped sob a mesma exclusão que publica active para evitar confirmar drenagem antes de novo pump.

## Contratos aceitos sem reabrir

Coordenador compartilhado entre manual/automático com RAII, limite três e exclusão por arquivo; relógio puro e intervalos limitados; lease validado na própria transação de persistir relatório; conta resolvida por subject não demo; evento/outbox atômicos por trigger ou transação operacional; supressão terminal por preferência e TTL explícito com cascatas; DispatchSubmitted sem alegação de OS/display/leitura; limites de notificação e autostart já registrados na pesquisa. Nenhuma alteração Google ou baseline aprovada pelo agente.

Gate1 permite integração OS somente após reconciliação dos achados e logs pelo principal. Reavaliação material fica limitada aos itens pendentes e riscos introduzidos pela correção; não reabrir desenho aceito sem mudança relevante.

## Reavaliação da remediação — tentativa 2 de 3

Fonte alterada revisada, limitada aos achados anteriores. Correções aceitas: replanejamento preserva o maior prazo de falha existente; fila manual e verificação manual direta respeitam deferral; finalização operacional mantém guard da geração e liberação de lease antigo usa somente Cancelled sem evento; falha ao criar worker remove active e libera CAS; publicação active do dispatcher revalida stopped sob mutex. Panic do sink vira falha de preparação retentável. Não se reabriram desenho/SQLite aceitos.

O principal recebeu evidência da lane store: sete testes do dispatcher com sink falso aprovados, inclusive preferência, conta/logout, envio terminal após restart, retentativa, parada e geração mantida durante submissão; duas integrações de disco aprovadas. Fonte dos testes foi inspecionada. Os dez testes focados core (oito iniciais mais duas regressões) aguardam resultado final no momento deste parecer.

**Parecer: aprovado para fase nativa condicionado somente ao resultado positivo dos dez testes core e reconciliação de seus logs pelo principal.** Não é necessário novo gate material para registrar esses resultados se não houver mudança de comportamento. Sem novo bloqueio de fonte. Esta aprovação não afirma execução nativa ou entrega Windows.
