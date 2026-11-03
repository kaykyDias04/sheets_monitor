# Gate 2 — shell, IPC e React, tentativa 1 de 3

2026-11-02. Revisão independente de app.rs e integração React, complementada pela inspeção estrutural do principal sobre desktop.rs/native.rs. Escopo: identidade da fila, comandos, eventos, listeners e sessão, startup Windows observado, ocultação e saída. Gate1 aceito não foi reaberto.

**Parecer inicial: aguardar correção de duas condições de interface/integração abaixo.** Nenhum bloqueio adicional de privacidade ou saída encontrado.

1. **P2 — pedido manual da conta atual pode ser descartado.** IPC valida geração e confirma entrada na fila, mas request_check guarda somente arquivo; tick limpa toda a fila ao inicializar/trocar identidade. Pedido novo aceito antes desse tick é apagado, em especial com monitoramento pausado logo após login. Guardar identidade/geração na fila ou sincronizar identidade ao enfileirar; descartar somente pedidos antigos. Provar novo pedido executando antes do primeiro tick ou após troca de conta, com pedido antigo descartado.
2. **P2 — coalescer refresh da visão principal.** MonitoringWorkspace preserva sinais recebidos durante sua leitura, mas onChanged chama refresh do OperationalApp, cujo run ignora pedido se locked. Um resultado de worker durante interação pode atualizar estado operacional e deixar summary/rows antigos até novo sinal. Preservar um pedido pendente e consumir após operação bloqueante; não adicionar polling. Teste com sinal durante leitura em andamento deve terminar com dashboard atualizado.

## Itens aceitos

- Serviço único e coordenador compartilhado; IPC valida geração/escopo/revisão antes de enfileirar; nenhuma nova aprovação de baseline.
- MonitoringWorkspace tem chave de geração/subject, listener cleanup inclusive registro assíncrono, e descarta resposta/toast antigo por envelope e sessão. Toast usa ID opaco; backend resolve na conta corrente antes de abrir. Não há URL arbitrária ou comando vindo do toast.
- Startup usa ação específica Windows, separado de preferências Google. HKCU exclusivo, caminho próprio citado, conflito preservado e estado de execução desconhecido. Capability continua core:default.
- Single-instance vem antes do setup; X oculta somente com bandeja utilizável. Fallback aprovado pelo principal mantém janela acessível. Sair é idempotente, usa callback fora da UI e ExitRequested bloqueia até exit_ready; limpeza limitada no integrador. Acesso Google não é necessário à saída.
- Payload limitado com nome/aba quando conhecida, diagnóstico/horário e gravidade; submissão ao plugin não é chamada de entrega.

## Evidência disponível

docs/evidence/sprint05/native-adapters-tests.md: check nativo aprovado, quatro testes de startup simulado e um de payload aprovados. Nenhuma notificação real ou entrada autostart alterada. Prova Windows de bandeja/janela oculta/instância/saída e eventual toast sintético explicitamente autorizado pertencem ao Gate3; não inferidas dos mocks.

## Reavaliação — tentativa 2 de 3

Revisadas somente as duas remediações e seus riscos. Fila manual agora guarda subject/geração; IPC e menu usam request_check_for dentro do guard existente, sem nova chamada auth.state reentrante. Tick cancela ativos antigos, atualiza identidade e retém exclusivamente pedidos da identidade corrente; pedido novo anterior ao primeiro tick deixa de ser eliminado. Wrapper de conveniência resolve auth antes do mutex de estado. Testes novos cobrem primeiro tick pausado, identidade anterior e descarte de geração antiga.

OperationalApp marca refresh pendente durante operação bloqueante e drena uma atualização ao liberar a operação, com mounted guard e referência do callback vigente. O teste de interação agora emite sinal enquanto bloqueado e exige a linha atualizada após conclusão. frontend-gate2.log registra **15 testes App aprovados**. Os cinco testes focados de adaptadores nativos continuam aprovados.

**Parecer: aprovado.** scheduler-gate2-tests.log foi reconciliado: **12 testes focados aprovados, 0 falhas**, incluindo as duas novas regressões de fila, em 1.79s; a integração de disco aparece filtrada nesse comando e não foi contada novamente. Nenhum outro bloqueio material introduzido pelas correções. Seguir para Gate3 de compilação, prova nativa e evidências, preservando ausência de toast/autostart real não autorizado.
