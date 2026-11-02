# Núcleo local e contrato de coleta

O núcleo Rust usa SQLite (`Store`) como fonte de verdade. Não contém acesso Google, não executa Apps Script e não envia dados para serviços externos. O adaptador coleta todas as respostas exigidas e somente então entrega `Snapshot` à transação de persistência.

## Cadastro e limites

Até 30 monitores ativos, com unicidade de ID de planilha, inclusive entre conexões SQLite diferentes. Um monitor contém nome, ID da planilha e lista explícita de IDs Apps Script autorizados. IDs de scripts não são descobertos implicitamente. Remover arquiva o monitor e libera a capacidade, preservando histórico e auditoria. Pausar invalida coletas em andamento. A configuração dos IDs é imutável na V1; para alterar, arquive e cadastre outro monitor, com novo baseline.

## Baseline e comparação

1. `begin_run(id)` adquire lease persistido com token único, por 15 minutos.
2. `complete_run(ticket, snapshot)` valida identidade da planilha/Drive e o conjunto exato de scripts autorizados; calcula indicadores estáticos e salva snapshot completo.
3. Sem baseline aprovado, a coleta é somente candidata. `preview_baseline(id)` retorna conteúdo, `snapshot_id` e fingerprint SHA-256.
4. A interface exige revisão e aprovação humana explícita. `approve_baseline(id, snapshot_id, fingerprint)` rejeita ID antigo, digest divergente, aprovação repetida e coleta ainda em andamento.
5. Após aprovação, cada coleta completa é comparada à coleta completa anterior. O baseline aprovado permanece preservado. Falhas de coleta não avançam esse cursor.
6. Snapshot, cada diferença, cada entrada da outbox e o cursor são gravados na mesma transação. Nenhuma diferença detectada é descartada por limite de volume ou gravidade.

O diff usa ponteiros JSON RFC 6901 e ordem determinística de chaves de objetos. Valores, fórmulas, formatação, estrutura, metadados, permissões, fontes e indicadores entram na comparação. Arrays são comparados por índice. Remoção/adição de uma subárvore produz uma diferença que inclui o conteúdo completo da subárvore. Reordenação de arrays pode gerar diferenças mesmo sem alteração semântica; a V1 não suprime esses eventos. Ausência de campo difere de JSON `null`.

Somente estados efetivamente coletados são comparados: mudanças e reversões entre duas coletas podem não ser observadas. Não há promessa de capturar cada edição individual do Google nem de operação em segundo plano com o aplicativo fechado.

## Notificações e falhas

Existe uma entrada durável na outbox para cada diferença. O aplicativo só confirma a entrega depois que seu adaptador de notificação aceita a operação. Falhas têm backoff exponencial (30 segundos a 1 hora), persistido entre reinícios, e podem ser reenfileiradas manualmente. Entrega é pelo menos uma vez: uma falha do processo após notificar e antes de confirmar pode repetir a notificação. O histórico continua disponível independentemente do sistema operacional.

`pending_notifications()` retorna somente itens vencidos. `notifications_all()` inclui entregues e pendentes, com tentativas, próximo horário e último erro de categoria fechada. Nunca se grava texto de exceção de provedor: `fail_run` aceita somente categorias conhecidas e substitui qualquer outra mensagem por `collection_failed`.

SQLite guarda conteúdo de planilhas e scripts em texto local; não contém tokens OAuth. A proteção em repouso desses dados depende das permissões do perfil Windows e da criptografia de disco do usuário. Não se compartilha o banco nem se versionam bancos reais. Modo demonstrativo e modo Google devem usar bancos distintos.

## Análise Apps Script V1

Cada fonte e manifest autorizado é preservado para diff. A análise gera indicadores determinísticos de HTTP externo, execução dinâmica, escrita/limpeza, e-mail, gatilhos, operações de arquivo e escopos/configuração web do manifest. É heurística estática: pode encontrar texto em comentários/strings e não resolve chamadas indiretas, execução efetiva, gatilhos instalados ou destinos em tempo de execução. Ausência de indicador não certifica segurança. Não há execução nem alteração de script.

## Evidência prevista

Testes unitários cobrem os limites, unicidade, aprovação exata, snapshots incompletos, fonte/manifest, diff, isolamento de falhas, leases, outbox e retomada por reabertura de banco. Os resultados efetivamente executados ficam no relatório de verificação em `docs/evidence/`; a presença dos testes no código não declara que foram executados.
