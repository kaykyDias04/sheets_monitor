# Probe Windows isolado — preparação e tentativas

Binário opcional: `src-tauri/examples/native_probe.rs`; implementação `src-tauri/src/native_probe.rs`. A feature `native-probe` permanece desligada por padrão e não adiciona comandos IPC de teste ao aplicativo operacional.

## Verificação executada

`cargo check --manifest-path src-tauri/Cargo.toml --features native-probe --example native_probe` aprovado sem avisos; última execução concluída em 3.28s. A primeira rodada identificou somente helpers de fixture não usados pelo probe; o módulo sintético recebeu tolerância de dead_code exclusivamente fora dos testes, sob a feature opcional. Compilar não comprova funcionamento de bandeja, janela, toast ou saída.

## Contrato da execução preparada

Após build do exemplo, executar o binário com `--report CAMINHO_ABSOLUTO_DO_JSON`. Comando de compilação: `cargo build --manifest-path src-tauri/Cargo.toml --features native-probe --example native_probe`. Os resultados efetivos ficam nas seções de tentativas abaixo.

O probe configura o identificador `com.sheetsmonitor.desktop.sprint05probe` antes do builder/single-instance, cria janela about:blank identificada como TESTE SINTÉTICO e usa diretório UUID temporário próprio. Não abre APPDATA/banco/cofre de produção, não solicita OAuth e não modifica autostart. A autenticação vem de vault sintético em memória; o transporte OAuth rejeita qualquer chamada. O executor injeta observações sintéticas no comparador real e persiste relatório com o lease corrente. Aprovações têm origem `synthetic:test`.

A sequência foi preparada para verificar serviço/coordenador compartilhado, primeira execução, X/close ocultando a janela através do shell real, execução com janela oculta, intervalo simulado de 60s, pausa sem execução, preferência de 300s, retomada, reabertura e request_exit com drenagem limitada. O JSON distingue callback programático de clique físico no menu: `physical_menu_clicks_verified` permanece false. O agente principal deverá registrar separadamente inspeção Windows, single-instance entre processos e clique real em Sair. Cenários 5/17 da matriz continuam `pending_native` até essa reconciliação.

O padrão envia **zero toasts**. `--test-toast` é opção separada, somente após autorização humana explícita; a fixture ativa a preferência local e o corpo contém TESTE SINTÉTICO. Depois de `DispatchSubmitted`, o processo mantém o loop nativo por 15s para permitir observação da tarefa assíncrona. Isso não comprova recebimento do Windows, exibição, clique ou leitura. Este registro não afirma autorização nem execução desse caminho. Permissão reportada pelo plugin desktop não comprova configurações Windows/DND.

O relógio injetado evita esperar 60/300s; a janela, bandeja e processo são reais somente quando o binário for efetivamente executado. A campanha com 30 planilhas e o instalador permanecem na Sprint06. O probe não consulta Google. Relatórios contêm somente dados sintéticos, indicadores técnicos, contagens e PID; nenhum ID/identidade/conteúdo Google real.

## Tentativas nativas e correção do manifest — 2026-11-03

O agente principal executou a primeira tentativa padrão: o loader encerrou o processo com código `-1073741511` (`0xC0000139`), antes de produzir relatório JSON. Os arquivos `native-probe-build.log`, `native-probe-stdout.log`, `native-probe-stderr.log` e `native-probe-dependencies.log` foram preservados. A inspeção do principal identificou import de `TaskDialogIndirect` e ausência de manifest no exemplo.

A correção acrescentou `src-tauri/native-probe.manifest` com dependência Common Controls 6 e execução `asInvoker`. O build script aplica `/MANIFEST:EMBED` e `/MANIFESTINPUT` somente aos exemplos, quando a feature opcional e o target Windows/MSVC estão ativos. O argumento para exemplos é suportado pelo [Cargo](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-arg-examples); dependências de assemblies são descritas pela [Microsoft](https://learn.microsoft.com/en-us/cpp/build/reference/manifestdependency-specify-manifest-dependencies?view=msvc-170). Nenhum componente Windows instalado foi alterado.

`cargo build --manifest-path src-tauri/Cargo.toml --features native-probe --example native_probe` passou em 56.13s, registrado em `native-probe-build-corrected.log`. O SDK Manifest Tool extraiu o recurso 1 com sucesso; manifest e saída estão em `native-probe-embedded-corrected.manifest` e `native-probe-manifest-corrected.log`.

Uma execução padrão corrigida foi autorizada e realizada, **sem `--test-toast`**. O processo chegou ao código Rust e terminou com código 1: `native_probe_failed: fixture_baseline`. Não houve relatório JSON nem prova de janela/bandeja. Evidências: `native-probe-stdout-corrected.log`, `native-probe-stderr-corrected.log` e `native-probe-exit-corrected.json`. O bloqueio era o guard que permitia origem `synthetic:test` somente sob `cfg(test)`; o exemplo opcional usa uma compilação normal. A correção de manifest não ampliou esse guard. O principal recebeu o diagnóstico. Nenhuma exibição de notificação é afirmada.

## Limite da autorização sintética opcional

O principal autorizou permitir `synthetic:test` sob `cfg(any(test, feature = "native-probe"))`. A feature permanece desligada por padrão; o aplicativo operacional não recebe origem por IPC e continua usando `human:native` no fluxo de aprovação humana. O probe usa exclusivamente sua conta fixture e banco UUID temporário. Nenhum guard de elegibilidade, revisão, escopo ou lease foi relaxado; nenhuma baseline de produção foi aprovada. Não foi usada origem humana fictícia. A nova compilação e tentativa sem toast serão registradas em arquivos com sufixo `-validated`, preservando todas as falhas anteriores.

## Execução nativa validada — sem toast

A compilação corrigida passou em 48.52s (`native-probe-build-validated.log`). O exemplo foi executado uma vez com `--report` e **sem `--test-toast`**, terminando com código 0 em 7.47s. `native-probe-exit-validated.json` registra o término e `native-probe-validated.json` contém os resultados do processo Windows real. Stdout/stderr separados estão nos arquivos com sufixo `-validated.log`.

Resultados observados: janela e bandeja disponíveis; close ocultou a janela; um check terminou enquanto ela estava oculta; três checks foram concluídos no total. Próximas execuções persistidas corresponderam a 60.000ms e 300.000ms do relógio injetado. Pausa não executou trabalho, retomada executou novamente, reabertura funcionou, o coordenador foi compartilhado e o callback de desligamento drenou de forma limitada. O processo terminou normalmente e o relatório marcou `probe_passed: true`.

Esta prova combina shell/processo Windows reais com conta, banco, baseline, observações e relógio sintéticos. O relatório registra zero chamadas Google, nenhum acesso ao banco/cofre de produção e nenhuma alteração de autostart. `toast_requested` permaneceu false, `toast_submission` foi `not_requested` e não há confirmação de exibição. Os callbacks foram exercitados programaticamente; clique físico no menu, clique em toast, segunda instância entre processos e homologação Google continuam separados. A saída programática percorreu `DesktopShell::request_exit`; isso não é alegação de clique humano em Sair. O principal reconciliará os cenários 5/17 da matriz com esses limites.

O stderr da execução validada contém dois avisos: recurso padrão de ícone da janela ausente no exemplo (`0x80070715`) e falha Chromium ao desregistrar `Chrome_WidgetWin_0` no encerramento (erro 1412). O processo terminou com código 0 e o tray foi criado. Esses avisos foram comunicados ao principal; não se afirma stderr limpo nem validação visual dos ícones do binário operacional a partir do exemplo. Nenhum componente do sistema foi alterado para suprimi-los.
