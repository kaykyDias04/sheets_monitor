# Sprint 05 — pesquisa nativa e contratos

Consulta em 2026-11-02, antes de implementar. Somente fontes primárias; pesquisa não é evidência de execução nativa.

## Versões e compatibilidade

O registro oficial crates.io retornou versões estáveis: [Tauri 2.12.5](https://crates.io/crates/tauri/2.12.5), [notification 2.8.0](https://crates.io/crates/tauri-plugin-notification/2.8.0), [autostart 2.7.0](https://crates.io/crates/tauri-plugin-autostart/2.7.0), [single-instance 2.5.5](https://crates.io/crates/tauri-plugin-single-instance/2.5.5). Todas declaram Rust mínimo 1.90. O ambiente respondeu `rustc 1.99.1`; o manifesto atual declara 1.88 e precisa ser alinhado se usar essas versões. Fixar versões efetivas no Cargo.lock e manter dependências desktop opcionais, sem impor Tauri aos testes sem desktop.

## Notificações

O [guia oficial](https://v2.tauri.app/plugin/notification/) limita a identidade Windows ao aplicativo instalado; desenvolvimento pode aparecer como PowerShell. Não gerar instalador nesta sprint. A [Microsoft](https://learn.microsoft.com/en-us/windows/win32/shell/enable-desktop-toast-with-appusermodelid) descreve o vínculo de identidade pelo atalho/AUMID. Não criar atalhos nem registrar identidade artificialmente para simular homologação de distribuição.

A [fonte publicada 2.8.0](https://github.com/tauri-apps/plugins-workspace/blob/notification-v2.8.0/plugins/notification/src/desktop.rs) é decisiva quando o guia diverge: `permission_state` e `request_permission` retornam Granted no desktop sem diálogo; isso não mede as preferências Windows nem Não Perturbe. `show()` prepara a carga e dispara trabalho em segundo plano; o resultado da chamada Windows interna pode ser descartado. Portanto `Ok(())` deve significar **dispatch_submitted**, nunca entregue, exibida, lida ou sequer aceitação comprovada pelo Windows.

Contrato: verificar preferência da conta ativa antes de despachar; registrar tentativa e resultado honesto no outbox; erro de preparação pode ser retentado com limite; envio submetido é terminal para evitar repetir alertas sem confirmação de exibição. Crash entre envio e confirmação no SQLite pode repetir uma tentativa: exatamente uma entrega externa não é garantida. Eventos distintos permanecem no SQLite. Ausência/supressão da notificação nunca derruba o monitoramento.

A mesma fonte implementa `on_action` no desktop, embora o guia ainda diga Actions somente mobile. Windows reporta clique apenas no toast visível, não no Action Center; registrar callback Rust antes de enviar. Usar somente identificador opaco interno da notificação; resolver conta/registro no backend antes de abrir os detalhes. Callback antigo após logout/troca de conta não deve abrir dados da conta anterior. Não usar URL ou comando vindo do payload. Toasts antigos não podem ser removidos no Windows por essa API; minimizar conteúdo sensível. Sem callback, histórico e bandeja continuam sendo caminhos de acesso.

O [conjunto default do plugin](https://github.com/tauri-apps/plugins-workspace/blob/notification-v2.8.0/plugins/notification/permissions/default.toml) expõe comandos amplos ao JavaScript. Preferir API Rust e manter capability `core:default`, sem adicionar `notification:default` ou `autostart:default` por conveniência. A preferência do produto continua explícita e desligada inicialmente; não solicitar nem emitir teste real automaticamente.

## Bandeja, instância e saída

O [tray oficial](https://v2.tauri.app/learn/system-tray/) requer feature `tray-icon`; menu e callbacks podem ficar inteiramente em Rust. Criar ícone/menu primeiro; ocultar `main` somente se o ícone existe. Decisão explícita do principal na fase nativa: falha de tray ou hide deve prevenir fechamento e manter a janela acessível; somente Sair encerra voluntariamente com limpeza. Menu: abrir, estado, pausar, retomar, sair. Abrir mostra, desfaz minimização e foca a mesma janela; estado/pausa/retomada usam o mesmo coordenador do IPC.

Registrar [single-instance primeiro](https://v2.tauri.app/plugin/single-instance/). A segunda instância apenas solicita abrir a janela existente e termina; não inicializa segundo scheduler, SQLite ou OAuth. Não registrar argumentos/cwd nos relatórios. A [API RunEvent](https://docs.rs/tauri/2.12.5/tauri/enum.RunEvent.html) permite interceptar saída; sair explicitamente deve sinalizar parada, impedir novos trabalhos, acordar o serviço e realizar limpeza limitada antes de `exit`, sem bloquear a thread UI indefinidamente. Fechar para tray não cancela serviço nem exige React ativo.

## Inicialização Windows

O [plugin autostart](https://github.com/tauri-apps/plugins-workspace/blob/autostart-v2.7.0/plugins/autostart/src/lib.rs) constrói gerenciador usando caminho do executável e consulta `is_enabled`; registrar plugin não habilita autostart. A dependência [auto-launch](https://github.com/zzzgydi/auto-launch/blob/master/src/windows.rs) opera registro Windows; seu modo dinâmico tenta sistema e depois usuário. O builder do plugin não expõe seleção CurrentUser. Preferir adaptador explicitamente por usuário se a versão resolvida confirmar essa limitação, evitando aplicar opt-in a outros usuários quando executado elevado. Conferir fonte resolvida antes de implementar; não elevar privilégios.

Autostart pertence à aplicação no Windows, não à conta Google. Consultar estado real separadamente das preferências por conta; não habilitar/desabilitar ao trocar conta ou sincronizar preferências antigas. Ação humana explícita modifica registro, consulta novamente e só então informa sucesso. Falha deve preservar/mostrar estado real e erro. Valor inicial desligado não autoriza remover registro existente de outra instalação. Não testar habilitação real sem checkpoint explícito; mocks comprovam contrato, consulta real somente leitura comprova estado observado.

## Evidência possível sem instalador

Compilar executável Windows; comprovar janela, ícone/menu, abrir pela bandeja, fechamento ocultando janela, segunda instância, pausa/retomada, saída e encerramento de processo. Provar serviço com relógio/executor simulados separadamente; para coleta Google contínua, somente escopo atual válido com baseline humana correspondente. Todos os cadastros pendentes significam zero recursos elegíveis, sem restaurar seleção antiga.

Teste de toast precisa ser explicitamente identificado, preferência apropriada e confirmação humana de exibição. Gravar separadamente submissão ao plugin, observação humana e eventual clique. Sem confirmação humana ou identidade instalada, registrar homologação visual/distribuída pendente. Não inferir leitura. Uma campanha final com 30 planilhas e instalador permanecem Sprint 06.

## Contrato público proposto após Gate 0

O principal confirmou baseline histórico preservado e autorizou somente backend antes do Gate1. Proposta para desenhar integração/UI; assinatura final será reconciliada com APIs das lanes:

```rust
struct DesktopStatus {
    tray_available: bool,
    notification_capability: NotificationCapability, // DesktopUnverified | Unsupported
    startup: StartupStatus,
    exiting: bool,
}
struct StartupStatus {
    registered: bool, // existe valor com o nome fixo da aplicação, mesmo se conflitante
    managed: bool,    // entrada pertence ao identificador e executável esperados
    os_execution: StartupExecution, // Unknown; não inferir execução no login
    error_code: Option<String>,
}
enum NativeAction { Open, CheckNow, Settings, Pause, Resume, Exit }
enum DispatchReceipt { DispatchSubmitted }
```

Comandos aprovados pelo principal: `desktop_get_status()`, `desktop_set_startup(enabled: bool)`, `desktop_exit()`. Abrir usa shell/menu. Não criar comando de teste de notificação na produção neste momento; teste real será checkpoint humano com preferência ativa e sessão atual, rotulado como teste e nunca no setup. Comandos retornam DTOs sanitizados; não retornam caminho do executável nem conteúdo do registro. Modificação autostart é ação explícita global da aplicação por usuário Windows, independente da preferência por conta Google; o campo legado `start_with_windows` precisa ser mostrado como estado observado ou descontinuado como fonte de verdade, sem efeitos colaterais no save genérico.

Shell sugerido: `desktop::install(app, callbacks) -> DesktopShell`, `DesktopShell::status()`, `DesktopShell::set_startup(bool)`, `DesktopShell::show_main()`, `DesktopShell::request_exit()`. `callbacks` contém closures para pausar/retomar/obter estado do mesmo serviço e desligar com limite; não instancia scheduler nem chama Google. `desktop::builder()` registra single-instance antes dos demais plugins. Principal mantém setup/app.rs; lane nativa entrega adaptadores e shell após Gate1. Evento `desktop-state-changed` comunica mudança local, sem polling React.

Integração exata aprovada para preparar a fase nativa:

1. Principal declara módulo desktop sob feature desktop em lib.rs; mantém FoundationState, comandos e setup app.rs. Lane nativa modifica somente novo desktop.rs, adaptador em notifications/native.rs, mod de notifications e Cargo.toml/lock. Nenhum código nativo antes do Gate1.
2. `desktop::builder() -> tauri::Builder<tauri::Wry>` retorna builder com single-instance primeiro e notification inicializado; autostart usa HKCU próprio, sem plugin dinâmico. A segunda instância chama somente `show_main` da primeira, sem carregar DB/cofre.
3. `DesktopCallbacks { action: Arc<dyn Fn(NativeAction) + Send + Sync>, shutdown: Arc<dyn Fn() + Send + Sync> }` permite shell chamar o coordenador já gerenciado. Ação `CheckNow` solicita os arquivos elegíveis pela fila, sem executar trabalho na callback UI; `Settings` mostra janela e emite navegação para preferências.
4. `desktop::install(&AppHandle, DesktopCallbacks) -> DesktopShell` tenta ícone/menu; falha é representada em tray_available sem abortar shell principal. A instância clonável fica em `app.manage`, status disponível via `State<DesktopShell>`. Principal chama install após serviço criado e antes de atender interação; sem ciclo de dependências entre módulo desktop e app.rs.
5. Hook de janela recebe fechamento de `main`, previne fechamento até exit_ready e oculta somente quando tray_available e !exiting. Sem tray ou se hide falhar, mostra e mantém a janela acessível. Saída voluntária explícita é idempotente: set exiting, invoca shutdown em thread auxiliar, aguarda no máximo contrato do serviço e chama AppHandle.exit; o hook ExitRequested também bloqueia saída enquanto a limpeza não estiver pronta. A janela não pode ficar invisível sem ícone após falha de hide/tray.
6. DTO público final usa String para notification_capability e os_execution, com valores fechados documentados (`desktop_unverified`/`unsupported`, `unknown`). `startup.registered` e `startup.managed` são fatos observados; valor divergente resulta em registered=true, managed=false e erro startup_entry_conflict, preservado. UI aprovada: “Registro ativo/desativado; execução definida também pelo Windows”; conflito tem diagnóstico próprio e não vira falsa indicação de ausência.

Notificação, contrato final alinhado com o dispatcher do principal: `NativeNotifier` implementa `NotificationSink::submit(&NotificationRecord) -> NotificationOutcome`, incluindo `DispatchSubmitted`, erro de preparação retentável ou Unsupported. Recebe evento já persistido, após claim durável e verificação de preferência/conta/geração. Dispatcher usa `auth.with_generation` para chamada curta de submissão, sem rede dentro do guard. A callback usa id opaco e resolve novamente registro/conta antes de emitir evento `notification-open` para detalhes; toast antigo não seleciona conta nem reautentica. DTO do evento deve incluir apenas id opaco e geração, com detalhes lidos pelo IPC corrente.

### Autostart por usuário sem ampliação de privilégio

Recomendação: adaptador pequeno exclusivo em `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run`, valor com nome fixo da aplicação, comando gerado pelo backend com caminho do próprio executável entre aspas e argumento fixo `--from-autostart`. Não aceitar caminho/argumentos do frontend. O [contrato Microsoft de Run](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys) distingue usuário/máquina e não garante início imediato; rejeitar comando acima de 260 caracteres. Usar crate já transitiva `winreg` ou API Microsoft apropriada, sem PowerShell subprocesso. Não modificar HKLM, StartupApproved ou entradas de terceiros. Se valor existente divergir do executável esperado, devolver conflito e preservar; não substituir/remover silenciosamente outra instalação.

Este contrato mede **registro de inicialização**, não execução futura ou override do Task Manager. UI: “Registrado para iniciar com Windows”; ajuda curta informa que o Windows pode desativar/adiar. Não declarar `blocked_by_windows=false` sem evidência. Sem modificar estado no teste, provar consulta real somente leitura; habilitar/desabilitar de verdade fica para checkpoint explícito. `--from-autostart` só oculta janela depois de tray criado e preferência de minimizar conhecida; falha de tray sempre mostra janela.

### Retenção e preferência — decisão do principal

Após Gate0, o principal definiu TTL efetivo do escopo inclusive pendências; expiração é explícita, e preferência desligada suprime terminalmente sem reenviar backlog quando ligada. Isso substitui a proposta inicial do store de preservar fontes indefinidamente por pendências. Aceito: é política coerente com retenção, desde que o estado `suppressed`/`expired` nunca seja chamado de entrega e o relatório registre a possível perda intencional do aviso. Eventos distintos continuam disponíveis durante seu TTL; reativar preferência vale para observações futuras. Testes devem provar que reativação não revive itens suprimidos/expirados, e que remoção de fonte/attempts não viola FKs nem cria outbox órfão. Um evento pendente pode vencer sem nunca ter sido exibido; esse limite é deliberado, não falha de integridade.

## Revisão após homologação negativa — 2026-11-03

A pesquisa do plugin e o plano acima são históricos. O primeiro teste visual autorizado foi submetido pelo plugin e o usuário informou que não apareceu. A análise e as evidências negativas foram preservadas em [notification-diagnosis.md](evidence/sprint05/notification-diagnosis.md).

Decisão vigente autorizada pelo principal: Windows usa diretamente a crate Microsoft windows e as APIs WinRT; a dependência notification foi removida, e desktop::builder registra somente single-instance, ainda primeiro. Desenvolvimento usa identidade PowerShell explícita com indicação no título, independente do diretório; release usa a identidade configurada, sem fallback após bloqueio. Show retorna erros observáveis, Failed enfileira recibos sem entrar em auth/SQLite, e Activated encaminha apenas ID interno para o guard de sessão existente. Os delegates são limitados a 128 entradas/quinze minutos; não há thread por toast. Callbacks terminais têm gate atômico de uma execução e protegem a fronteira COM de panic Rust.

Readiness é API separada somente leitura, sem Show. O probe com operação identificada comprovou CreateToastNotifierWithId bem-sucedido para a identidade PowerShell instalada, mas Setting retornou ElementNotFound (80070490). Readiness mantém setting=unknown e erro com prefixo native_setting_hresult. A autorização técnica permite tratar somente esse erro específico do getter como desconhecido, sem impedir uma tentativa já autorizada pela preferência/usuário; Show e Failed continuam verificando erros reais. Valores Disabled conhecidos sempre bloqueiam, e outros erros do getter geram retentativa. Não foi alterada identidade do processo, atalho, registro de identidade ou configuração Windows. Outra tentativa visual requer nova autorização específica.
