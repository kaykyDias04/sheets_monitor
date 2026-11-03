# Falha do primeiro toast e correção do adaptador Windows

2026-11-03. O teste sintético anteriormente autorizado submeteu um alerta pelo plugin; `native-toast-submission.json` preserva esse resultado como dispatch_submitted, sem confirmação visual. O usuário confirmou **não apareceu**. Essa evidência negativa não foi substituída por testes unitários ou por uma nova tentativa. SM-023 não pode ser declarado homologado visualmente por esse resultado.

## Causa técnica identificada

A fonte local publicada de tauri-plugin-notification 2.8.0, `src/desktop.rs`, decide a identidade Windows por sufixo do diretório do executável. Somente `target/debug` e `target/release` recebem o fallback de desenvolvimento. `target/debug/examples` e `target/x86_64-pc-windows-msvc/debug` recebem o identificador configurado, embora não sejam aplicativos instalados. O probe usou sua identidade própria sem registro de distribuição. O comportamento divergente por caminho foi confirmado no código; sem erro Windows observável, não se atribui causalidade adicional a Não Perturbe ou às configurações globais.

Além disso, show_tracked apenas cria uma thread; a chamada Windows ocorre depois e seu erro pode ser descartado. O callback atual do plugin não permitia observar esse erro. A dependência notify-rust 4.18.2 usa o AppUserModelID PowerShell somente quando app_id está ausente, confirmando que o caso do probe não selecionava esse caminho.

A heurística incorreta comprova a escolha de identidade; afirmar que foi a causa única da ausência visual seria uma inferência sem HRESULT do envio original. O primeiro probe somente leitura do adaptador corrigido retornou native_hresult_80070490, ainda sem distinguir criação de notifier da consulta Setting. Get-StartApps confirmou que a identidade PowerShell exata existe neste Windows. A evidência original winrt-probe-readiness.json foi preservada; a rodada diagnóstica seguinte distingue native_create_hresult de native_setting_hresult sem enviar alerta. Código 80070490 não é convertido em afirmação de bloqueio de permissão.

A rodada diagnóstica autorizada foi executada uma vez: build explícito MSVC do example aprovado em 2m54s; processo terminou com código 0 em 4.7s. `winrt-probe-readiness-stage.json` comprova **CreateToastNotifierWithId bem-sucedido e falha somente em Setting**, com native_setting_hresult_80070490. setting permanece unknown. toast_requested=false, toast_submission=not_requested; nenhuma tentativa visual nova ocorreu. O probe preservou conta/cofre/banco reais e autostart, com três verificações sintéticas, uma oculta, e parada limitada. Os logs próprios dessa rodada foram preservados.

Fontes publicadas: [plugin desktop 2.8.0](https://github.com/tauri-apps/plugins-workspace/blob/notification-v2.8.0/plugins/notification/src/desktop.rs), [wrapper Windows do Tauri](https://github.com/tauri-apps/winrt-notification). Os arquivos exatos resolvidos foram lidos no cache Cargo, sem alteração das dependências.

## Correção autorizada, sem enviar outro alerta

O adaptador agora usa diretamente a crate Microsoft windows 0.62.2, já presente transitivamente, com APIs WinRT:

- Política de identidade explícita por build: desenvolvimento usa a identidade PowerShell documentada no wrapper, com título “[Desenvolvimento]”; release usa somente a identidade configurada. Nenhuma alternativa é selecionada após bloqueio de permissão. Nenhum diretório do executável determina identidade.
- [ToastNotifier.Setting](https://learn.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotifier.setting) é consultado antes do envio; bloqueio observado resulta em permission_blocked. Essa propriedade não comprova apresentação na tela.
- [ToastNotifier.Show](https://learn.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotifier.show) é chamado sincronamente e seu HRESULT de erro é sanitizado como native_hresult_XXXXXXXX. Retorno sem erro permanece dispatch_submitted, sem inferência de exibição/leitura.
- [Failed](https://learn.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotification.failed) recebe falhas posteriores e entrega somente o código sanitizado a uma fila de recibos do serviço; callback não pode adquirir auth/SQLite durante Show. [Activated](https://learn.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotification.activated) encaminha ID interno opaco; o integrador mantém validação da conta/geração antes de abrir.
- Delegates/toasts têm retenção limitada a 128 entradas e quinze minutos; entradas concluídas são removidas no próximo envio. Não há thread por toast. Capacidade esgotada gera retentativa durável, sem remover silenciosamente handler válido. Acesso por histórico continua disponível após expirar a retenção de callbacks.
- DOM de XML cria elementos/textos; nomes e diagnóstico não são interpretados como markup, comando ou URL. COM inicializa uma vez por thread e usa STA já existente quando aplicável.

API somente leitura `native::readiness(&AppHandle)` retorna identity_mode, setting e error_code; não chama Show. Não foram criados atalhos, instalador, registro de identidade, elevação ou alteração de configuração Windows. A identidade própria de distribuição continua dependendo da Sprint06.

## Verificação executada

`cargo test --manifest-path src-tauri/Cargo.toml --lib notifications::native:: -- --nocapture`: **4 aprovados, 0 falhas, 247 filtrados**, execução 0.00s, compilação 34.68s. Prova payload limitado, política de identidade independente de diretório, mapeamento de configuração e HRESULT sanitizado. A primeira tentativa de check identificou import relativo do módulo incorreto; corrigido antes do resultado acima.

Após acrescentar um caso de conteúdo hostil para o DOM, mesmo comando: **5 aprovados, 0 falhas, 252 filtrados**, 0.01s, compilação 9.18s. O teste novo inicializa COM e cria somente XML em memória; comprova que fragmentos de texto não criam elementos audio ou markup. Não cria notifier nem chama Show. A diferença de total filtrado inclui testes novos das outras lanes no workspace compartilhado.

Rodada final após gates atômicos dos callbacks, remoção do plugin e exceção estrita para o getter Setting: mesmo comando, **6 aprovados, 0 falhas, 252 filtrados**, 0.01s, compilação 1m48s. A regressão nova permite somente ElementNotFound como estado desconhecido, preserva os quatro valores Disabled e recusa outros erros, incluindo AccessDenied. Não chama Show. Cargo.lock foi resolvido por cargo metadata sem atualizar versões alheias; dependências exclusivas do plugin antigo foram removidas.

Nenhum desses testes envia toast. A próxima submissão real exige uma nova autorização específica do usuário após revisão da correção. Até haver confirmação humana, a exibição permanece pendente.
