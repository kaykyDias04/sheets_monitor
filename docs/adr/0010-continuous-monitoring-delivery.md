# ADR 0010 — Monitoramento contínuo e limites de entrega

Estado: aceito para a Sprint 05. Escopo: SM-022 a SM-025.

O serviço Rust será o proprietário do relógio, da fila e da concorrência. Operações manuais e periódicas compartilharão o limite de três verificações e a exclusão por arquivo. Controle persistido, geração da sessão, revisão do cadastro e lease transacional impedem que trabalhos cancelados publiquem observações antigas. O primeiro início exige a ação explícita de iniciar; reinícios preservam essa escolha. Reinício e suspensão escalonam trabalho, sem tentar reconstruir alterações intermediárias que não foram observadas.

Eventos de integridade e suas notificações serão gravados na mesma transação SQLite. Falhas operacionais terão eventos próprios e não produzirão violações estruturais artificiais. Cada evento distinto terá seu registro, mesmo quando a apresentação agrupar alertas.

O retorno de sucesso do plugin nativo significa submissão ao plugin. Não comprova aceitação pelo Windows, exibição nem leitura. O histórico usará `DispatchSubmitted`; uma interrupção entre submissão e confirmação local poderá produzir tentativa ambígua e repetição física. A aplicação não promete exatamente uma exibição nativa. Configurações do Windows e Não incomodar exigem comprovação visual separada.

Notificações ficam desativadas por padrão. Eventos detectados enquanto essa preferência estiver desligada serão registrados como suprimidos, sem envio retrospectivo surpresa quando ela for ativada. Falhas de envio elegíveis poderão ser repetidas até o prazo efetivo do histórico do escopo. Ao expirar, a pendência será tratada como expirada, nunca como entregue; a limpeza seguirá a retenção configurada, de 90 dias por padrão. Manter pendências para sempre permitiria crescimento ilimitado do banco. A exclusão explícita das atividades também removerá seus registros de notificação e tentativas.

A inicialização com Windows é uma configuração real do aplicativo para o usuário Windows atual, independente da conta Google. Só será alterada por uma ação explícita. Uma preferência antiga da conta não poderá ativar registro do sistema silenciosamente. Não serão removidas instalações ou entradas de outros programas. A janela só será ocultada no fechamento após confirmar a existência da bandeja; caso contrário continuará acessível. Sair cancela trabalho e aguarda um prazo limitado fora da thread da interface; SQLite e leases protegem a retomada caso uma requisição ainda esteja drenando.

O instalador e a identidade final das notificações instaladas pertencem à Sprint 06. Evidências desta etapa distinguirão testes sintéticos, execução nativa do binário de desenvolvimento, leitura Google autorizada e confirmação humana.
