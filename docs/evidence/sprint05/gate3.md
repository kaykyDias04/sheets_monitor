# Gate 3 — reconciliação final da Sprint 05

Data: 2026-11-03. Escopo: evidências finais, relatório `docs/sprint-05-report.md` e deltas do adaptador Windows/IPC de notificações desde a revisão focada. Os algoritmos inalterados aprovados nos Gates1/2 não foram reabertos. Nenhum alerta, escrita Google, baseline, configuração Windows ou processo do usuário foi alterado por esta revisão.

**Parecer: APROVADO para o encerramento técnico local da Sprint05, com os limites de homologação abaixo preservados. Sem achado material bloqueador.** O aplicativo normal permanece aberto para uso do usuário. O principal pode concluir os registros e commits locais, encerrando a execução sem iniciar a Sprint06.

## Evidências reconciliadas

- O primeiro alerta autorizado não apareceu; `native-toast-observation.json` conserva esse resultado negativo. O usuário autorizou uma única tentativa corrigida e confirmou **Sim, apareceu**, registrado em `native-toast-winrt-observation.json`. O remetente de desenvolvimento Windows PowerShell foi informado. Há duas tentativas autorizadas nesta sprint; nenhuma tentativa adicional foi necessária para completar a documentação.
- `native-toast-winrt-run.json` registra interrupção `0xc000013a`, sem JSON final do harness e sem coleta completa de callbacks. A origem do sinal não foi estabelecida. A confirmação humana estabelece exibição daquele alerta sintético; não estabelece clique, leitura, coleta de Failed/Activated ou saída limpa desse processo específico. O relatório mantém essa distinção.
- As evidências anteriores sem toast (`native-probe-validated.json`) e do aplicativo real (`native-qa.json`) comprovam separadamente execução oculta com recursos sintéticos e saída voluntária limitada. Não foram reutilizadas como prova de término limpo da tentativa interrompida.
- `winrt-final/rust-test.log`:257 unitários aprovados, duas integrações aprovadas, um teste de cofre ignorado por padrão. `winrt-final-frontend.log`:50 testes em10 arquivos. Logs finais de Clippy, lint, typecheck, formato e compilação MSVC estão presentes; `winrt-final/rust-build.log` e `final-corrected/rust-build.log` terminam em compilação aprovada. Nenhuma nova execução foi feita apenas para repetir essa evidência.
- Os20 cenários da matriz distinguem testes sintéticos, execução Windows, leituras Google históricas e limites de interação física. Todos os nomes de testes listados foram encontrados no log Rust final. O teste sintético herdado de30 workbooks não é apresentado como campanha final Google de30 arquivos.

## Delta de fonte e autorização

`permits_submission` admite apenas Enabled ou o HRESULT específico80070490 do getter Setting, após criação bem-sucedida do notifier. O HRESULT é representado como configuração desconhecida, sem inferir permissão concedida. Todos os valores Disabled conhecidos continuam bloqueados; outros erros seguem o tratamento de retentativa. No fluxo de produção, a tentativa continua condicionada à preferência explícita do produto, conta/geração correntes e claim durável, com Show/Failed observáveis. O alerta sintético isolado depende da autorização humana e flag próprias do harness. Não há fallback de identidade para contornar uma negação nem alteração de configuração Windows.

`notification_get_readiness` consulta disponibilidade fora da UI e não chama Show. `notification_retry_blocked` exige ação IPC explícita, verifica bloqueios conhecidos e valida expected_generation antes da operação local. O store reabre somente permission_blocked da conta corrente quando a preferência está ativa e o TTL ainda é válido; expiração ocorre antes da atualização. Registros suprimidos pela preferência ou expirados não são ressuscitados. A consulta e o reenvio não alteram credenciais, autorização Google ou registro Windows.

As conclusões anteriores sobre a fila não reentrante de Failed, CAS por conta/tentativa, preservação de submitted_ms, retenção e isolamento de callbacks permanecem em `gate3-notifications.md`. A condição visual daquele parecer foi posteriormente satisfeita pela confirmação humana; não houve confirmação de clique ou leitura.

## Limites que continuam explícitos

A identidade própria instalada, clique físico no toast/menu da bandeja, execução autostart no login e campanha integrada final pertencem à homologação pertinente; nenhum instalador ou campanha final foi executado aqui. O usuário não teve autostart habilitado automaticamente. O console observado no lançamento do build de desenvolvimento não é apresentado como comportamento homologado da distribuição final.

Os quatro cadastros reais observados estavam pendentes, com zero escopos atuais elegíveis. Não houve coleta Google contínua real nova, restauração de seleção antiga ou aprovação de baseline pelo agente para viabilizá-la. Recursos sintéticos ficaram isolados e identificados. Evidências históricas das sprints anteriores permanecem históricas.

Não há CI remoto ou PR publicado afirmados. A aprovação deste gate cobre a implementação e as evidências técnicas locais, sem substituir o registro final de progresso/commit pelo principal. Após esse registro, aguardar o Prompt06.
