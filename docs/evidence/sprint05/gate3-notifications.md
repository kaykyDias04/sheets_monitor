# Gate 3 — revisão focada da correção de notificações

Data: 2026-11-03. Revisão independente de fonte, limitada às alterações novas em `notifications/native.rs`, `notifications/windows.rs`, fila de falhas em `notifications/service.rs` e `Database::retry_native_submission`. Os algoritmos inalterados dos Gates1/2 não foram reabertos. Nenhum toast, alteração de registro, configuração Windows ou chamada Google foi executado por esta revisão.

**Parecer de fonte: aprovado, sem achado material P1/P2 novo nos caminhos revisados. A aprovação funcional de SM-023 continua condicionada à homologação nativa real; esta revisão não a substitui.**

## Invariantes verificados

- A identidade decorre da política explícita do build: desenvolvimento usa identidade PowerShell com rótulo de desenvolvimento; release usa o identificador configurado. Não há heurística por diretório do executável nem troca de identidade para contornar uma negação.
- Readiness chama apenas inicialização COM, criação do notifier e leitura de Setting. O caminho não chama Show nem modifica preferências Windows. Erros recebem HRESULT sanitizado; o código atual diferencia os estágios create/setting, enquanto a evidência inicial ainda contém o prefixo anterior.
- Payload vira nós de texto XML, com limites de tamanho e remoção de controles; não é interpretado como URL, comando ou marcação. Show bem-sucedido permanece dispatch_submitted, sem inferir exibição/leitura.
- A geração de autenticação continua protegida durante a submissão. Failed pode ocorrer dentro de Show: a callback apenas coloca um recibo em uma fila protegida por mutex curto e acorda o worker. Não abre SQLite nem reentra no mutex de autenticação. A reconciliação remove o lote da fila antes de adquirir o guard de geração/DB.
- Falha de banco na reconciliação devolve o recibo atual e os restantes à fila. Logout/troca de geração descarta callbacks antigos conforme o contrato adotado, sem modificar o diário de outra sessão. O fato de uma submissão anterior não é transformado em confirmação de exibição.
- A reconciliação transacional usa conta, ID da notificação e tentativa exata. Callback antigo não cancela uma tentativa substituta; preferência desligada e expiração prevalecem sobre retry. submitted_ms é preservado como fato histórico da solicitação. Suprimido/expirado não é ressuscitado.
- Os delegates são mantidos em tabela limitada a128 entradas; callbacks de conclusão usam CAS para executar uma vez. A limpeza por TTL de15min é oportunista nas próximas submissões, não um temporizador que remove delegates exatamente aos15min. O limite de entradas permanece válido durante inatividade.
- Activated envia somente ID opaco ao callback de produção. O caminho de abertura existente despacha trabalho fora do callback e resolve novamente o registro para a conta conectada, sob a geração corrente. O shutdown continua com espera limitada fora da UI; esta mudança não adiciona joins ou espera de autenticação dentro de Failed.

## Evidências inspecionadas e limite da conclusão

`native-failure-dispatcher-requeue-corrected-tests.log`:12 testes aprovados, incluindo Failed síncrono sem deadlock, recibo tardio, logout, preferência, shutdown e requeue após erro de banco. `native-failure-store-tests.log`:5 testes aprovados para CAS, ordem de confirmação, preservação da submissão, supressão e expiração. Esses resultados foram inspecionados; nenhum teste foi reexecutado por esta revisão de fonte.

`native-toast-observation.json` preserva a única tentativa autorizada anterior: o usuário respondeu **Não apareceu**; não houve aceitação pelo OS, exibição ou leitura comprovada. `winrt-probe-readiness.json` é uma observação somente leitura do novo backend: setting unknown, HRESULT80070490, toast_requested=false. Sua aprovação do probe refere-se às invariantes do harness, não à disponibilidade das notificações.

Não existe autorização para um segundo alerta neste estado. Uma tentativa futura depende de autorização própria e deve registrar separadamente readiness, Show, eventual Failed e observação humana. O harness atual chama submit com callback de falha vazio; para usar um futuro teste como diagnóstico de falha assíncrona, a lane proprietária deve capturar Failed via submit_with_failure. Isto é uma observação sobre a evidência futura, sem achado bloqueador no dispatcher de produção.

## Snapshot das fontes revisadas (SHA256)

- native.rs: `65452D385613C5DB4E16200324C53A6DBF9037E06A498B6B40A1B5253B7544A7`
- windows.rs: `7F00E2E6BDE018130A7A2D01876B33D73D720518AA01A9B16F12F3ACBA6C173C`
- service.rs: `A473322D43DDC4B5E03B3876094903AAE142948CB1F8E4DE8D445B1E4DB0853E`
- monitoring_store.rs: `EE4CDFA0F0F5C27BD48133A8161BED6F4134F7D07B74D91A11ACD1E0DFD2753F`
