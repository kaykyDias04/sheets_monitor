# Sheets Monitor — integridade estrutural

Aplicativo Windows 11 x64 em Tauri 2, React/TypeScript/Vite, Tailwind CSS, componentes shadcn/Radix, Rust e SQLite.

A **Sprint 05 está em validação**: monitoramento contínuo Rust, notificações, bandeja e recuperação de falhas. [Relatório da etapa](docs/sprint-05-report.md). Sprints 01–04 e suas evidências foram preservadas. A Sprint 06, com homologação final em escala e instalador, exige nova autorização.

O fluxo diário usa a conta Google conectada e dados do backend. O cadastro admite até 30 arquivos, incluindo pausados; a descoberta não limita a lista a 30. Cada proteção exige abas, regras e uma baseline correspondente aprovada pelo usuário. Nenhuma operação altera arquivos Google ou executa Apps Script.

Inicie o monitoramento na visão geral. O serviço verifica arquivos elegíveis a cada 60–300 segundos, com até três verificações concorrentes, mesmo com a janela oculta. Fechar oculta para a bandeja quando disponível; use **Sair** para encerrar. Autostart é opt-in por usuário Windows e permanece desligado até uma ação explícita. Notificações também ficam desligadas por padrão.

O mantenedor provisiona o cliente OAuth Desktop da aplicacao antes da distribuicao; o usuario comum apenas entra com Google. O modo avancado serve ao desenvolvimento autorizado. Sem provisionamento, o aplicativo informa essa pendencia, sem fabricar integracao ou exigir credenciais Cloud no fluxo diario. Publicacao Google e homologacao local sao coisas distintas.

Configurar a conexão local seguindo [OAuth](docs/oauth-setup.md). Nunca enviar senhas, secrets ou tokens no chat ou em commits. Drive usa opt-in de metadados; nenhuma permissão de escrita é solicitada.

## Executar

Pré-requisitos e versões em [docs/environment.md](docs/environment.md). Para configurar/verificar o ambiente:

```powershell
.\scripts\bootstrap.ps1
.\scripts\doctor.ps1
```

Instalar dependências e abrir a prévia de desenvolvimento:

```powershell
npm ci
npm run dev
```

A prévia operacional no navegador informa que o desktop é necessário e não simula uma conta conectada. Para abrir o aplicativo Tauri durante o desenvolvimento:

```powershell
. .\scripts\environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
npm run tauri dev
```

Para compilar e executar o desktop com arquivos estáticos incorporados, sem precisar do Vite em execução:

```powershell
npm run build
.\scripts\check-rust.ps1 -Task build
.\src-tauri\target\x86_64-pc-windows-msvc\debug\sheets-monitor.exe
```

O build de fundação usa `custom-protocol`, incorpora `dist/` e não gera instalador. O produto não exige servidor próprio ou webapp hospedado. npm e seu lockfile são o caminho reproduzível desta sprint; pnpm está disponível no ambiente, sem migração implícita do lockfile.

## Dados locais e limites

O desktop armazena dados operacionais por conta em `%APPDATA%\com.sheetsmonitor.desktop\foundation-v1.db`. As fixtures históricas da fundação ficam separadas e não aparecem como monitores reais. Bancos do protótipo não são alterados. Tokens e credenciais permanecem no cofre Windows, fora do SQLite e do renderer.

Online/Offline são resultados do motor; falhas de conexão, permissão ou API são Indeterminadas. Eventos distintos e sua fila de alertas são persistidos atomicamente. O histórico tem retenção de 90 dias por padrão, ajustável por proteção. Polling compara somente observações efetivas: não garante capturar edições intermediárias durante pausas ou indisponibilidade.

O banco possui 29 tabelas de produto após a migração 0005, que preserva as anteriores e acrescenta controle, agendas, eventos operacionais, outbox e tentativas. Não compartilhe bancos reais, relatórios privados ou credenciais em issues/evidências.

Um alerta marcado **Submetida ao plugin** não comprova exibição ou leitura no Windows. Não incomodar e preferências do sistema podem silenciá-lo. Alertas desligados são registrados como suprimidos, sem envio retroativo ao ativar; pendências também respeitam a retenção. Uma interrupção entre submissão e confirmação local pode provocar repetição visual. A identidade final da aplicação instalada será validada na Sprint 06.

## Verificar

```powershell
npm run lint
npm run format:check
npm run typecheck
npm test
npm run build
.\scripts\check-rust.ps1 -Task format-check
.\scripts\check-rust.ps1 -Task test
.\scripts\check-rust.ps1 -Task clippy
.\scripts\check-rust.ps1 -Task build
node scripts/verify-repository.mjs
```

Resultados executados e limitações ficam em [docs/progress.md](docs/progress.md) e [docs/evidence/sprint05](docs/evidence/sprint05/). A presença de código, workflow ou fixture não comprova integração real ou execução do CI hospedado.

## Documentação

- [Especificação original](docs/specification-v1.0.original.md), [requisitos operacionais](docs/requirements.md) e [aceite](docs/acceptance.md).
- [Backlog SM-001 a SM-028](docs/backlog.md), [arquitetura](docs/architecture.md) e [riscos](docs/risks.md).
- [SQLite](docs/database.md), [desktop](docs/desktop.md) e [ADRs](docs/adr/README.md).
- [Contrato de execução](AGENTS.md), [contribuição/GitHub](CONTRIBUTING.md), [segurança](SECURITY.md) e [licenciamento](docs/licensing.md).

O protótipo anterior está preservado em `archive/prototype/` e no commit `ffcaa49`. Ele está fora da aplicação ativa e não define o aceite desta sprint. Os IDs oficiais do anexo não foram renumerados.

## Publicação e licença

Código próprio sob [MIT](LICENSE); dependências mantêm suas licenças. Não há remoto GitHub autorizado/configurado: o trabalho permanece local e a instrução mínima de autenticação está em [CONTRIBUTING.md](CONTRIBUTING.md). Nada foi publicado ou enviado ao Google.

Ao encerrar a Sprint 04, aguardar o Prompt 05. Não avançar automaticamente.

Documentação da integração: [APIs](docs/google-apis.md), [contratos IPC](docs/api-contracts.md), [segurança](docs/security.md) e [testes](docs/testing.md).

Motor: [integridade](docs/integrity-engine.md), [baselines](docs/baselines.md), [fórmulas](docs/formula-analysis.md), [Apps Script](docs/apps-script-dependencies.md) e [eventos](docs/event-model.md).

## Guias de operacao

- [Interface e estados](docs/interface.md).
- [Descoberta de planilhas](docs/spreadsheet-discovery.md) e [gerenciamento](docs/spreadsheet-management.md).
- [Regras](docs/integrity-rules-ui.md) e [baselines](docs/baseline-ui.md).
- [Apps Script](docs/apps-script-ui.md) e [OAuth](docs/oauth-setup.md).

A referencia real aprovada cobre uma aba e duas celulas. A selecao atual de28 abas permanece em Configuracao pendente ate revisao das regras e aprovacao humana correspondente. Nenhum arquivo Google foi alterado pelo agente. Scheduler e notificacoes Windows continuam para a Sprint 05.
