# SM-022–SM-025: monitoramento contínuo e notificações Windows

O aplicativo passa a verificar proteções elegíveis em segundo plano com fila Rust, intervalo 60–300s, máximo global de três verificações e exclusão por arquivo. Eventos e outbox são atômicos; retentativas respeitam cotas, Retry-After, sessão, configuração e retenção. Bandeja, instância única e saída limitada preservam a execução oculta.

O adaptador WinRT observa erros imediatos/tardios e mantém ativação vinculada ao evento da conta atual. O primeiro teste visual falhou com o plugin anterior; após a correção, o usuário confirmou o alerta sintético. As duas observações permanecem registradas. Autostart é opt-in exclusivo HKCU e não foi habilitado pelo agente.

Validação: 257 unitários Rust, duas integrações e 50 React aprovados; Clippy/lint/tipos/formato e build Windows verificados conforme [relatório](sprint-05-report.md). Probes Windows reais comprovaram checks com janela oculta usando recursos sintéticos, pausa/retomada e saída. Aplicativo real preservou quatro cadastros pendentes e zero escopos elegíveis; nenhum Google write ou baseline real aprovado pelo agente.

Limites: identidade instalada, login autostart, clique físico em menu/toast e campanha final de 30 arquivos permanecem separados. Sem remoto autorizado, este é um rascunho local. Para publicação futura: autenticar com `gh auth login` e informar o remoto autorizado; nenhuma credencial deve ser enviada pelo chat. A Sprint 06 aguarda seu prompt.
