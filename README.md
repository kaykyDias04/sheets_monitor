# Sheets Monitor — integridade estrutural

Aplicativo Windows 11 x64 em Tauri 2, React/TypeScript/Vite, Tailwind CSS, componentes shadcn/Radix, Rust e SQLite.

A autorização atual é a **Sprint 03**, motor de integridade sob demanda: snapshots, baselines com aprovação humana, comparações estruturais/conteúdo/fórmulas, análise AST Apps Script, relatórios e eventos deduplicados. Aceite técnico concluído; cobertura real restrita e mutações reais pendentes estão no [relatório final](docs/evidence/sprint03/final-report.md). Aguardar Prompt 04. Mutações Google e aprovação de baseline real não são automáticas.

A Sprint 02 continua homologada com login, sessão/refresh e leituras reais das três APIs: [evidência](docs/evidence/sprint02/google-live.json). A tela Integração Google e a nova Integridade técnica usam backend real; navegador sem Tauri não simula sucesso. Dashboard, Planilhas, Histórico e preferências ainda conservam a demonstração da fundação. Gerenciamento definitivo, scheduler, notificações, bandeja, autostart e instalador aguardam suas etapas.

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

A prévia no navegador usa exemplos em memória. Para abrir o aplicativo Tauri durante o desenvolvimento:

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

No desktop, as quatro telas consultam fixtures SQLite em `%APPDATA%\com.sheetsmonitor.desktop\foundation-v1.db`. O seed é explícito e idempotente. Preferências persistem ao reiniciar; o navegador usa uma sessão em memória. Bancos do protótipo não são alterados ou migrados.

Os exemplos ilustram Online, Offline, Verificando e Indeterminado. Nenhum desses estados veio de uma consulta Google. O intervalo de 60–300 segundos e os 90 dias de histórico são parâmetros preparatórios; não há scheduler nem limpeza automática nesta sprint. Autostart permanece indisponível.

O banco possui 22 tabelas de produto: onze da fundação, quatro de integração e sete do motor na migração 0003. Tokens, senhas e segredos não fazem parte do esquema. Não compartilhe bancos reais, relatórios privados ou credenciais em issues/evidências.

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

Resultados executados e limitações ficam em [docs/progress.md](docs/progress.md) e [docs/evidence/sprint03](docs/evidence/sprint03/). A presença de código, workflow ou fixture não comprova integração real ou execução do CI hospedado.

## Documentação

- [Especificação original](docs/specification-v1.0.original.md), [requisitos operacionais](docs/requirements.md) e [aceite](docs/acceptance.md).
- [Backlog SM-001 a SM-028](docs/backlog.md), [arquitetura](docs/architecture.md) e [riscos](docs/risks.md).
- [SQLite](docs/database.md), [desktop](docs/desktop.md) e [ADRs](docs/adr/README.md).
- [Contrato de execução](AGENTS.md), [contribuição/GitHub](CONTRIBUTING.md), [segurança](SECURITY.md) e [licenciamento](docs/licensing.md).

O protótipo anterior está preservado em `archive/prototype/` e no commit `ffcaa49`. Ele está fora da aplicação ativa e não define o aceite desta sprint. Os IDs oficiais do anexo não foram renumerados.

## Publicação e licença

Código próprio sob [MIT](LICENSE); dependências mantêm suas licenças. Não há remoto GitHub autorizado/configurado: o trabalho permanece local e a instrução mínima de autenticação está em [CONTRIBUTING.md](CONTRIBUTING.md). Nada foi publicado ou enviado ao Google.

Ao encerrar a Sprint 03, aguardar o Prompt 04. Não avançar automaticamente.

Documentação da integração: [APIs](docs/google-apis.md), [contratos IPC](docs/api-contracts.md), [segurança](docs/security.md) e [testes](docs/testing.md).

Motor: [integridade](docs/integrity-engine.md), [baselines](docs/baselines.md), [fórmulas](docs/formula-analysis.md), [Apps Script](docs/apps-script-dependencies.md) e [eventos](docs/event-model.md).
