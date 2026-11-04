# Instalação do candidato Sheets Monitor V1.0

Estado: candidato local em homologação; não aprovado para distribuição a terceiros. O relatório da etapa e os hashes efetivos ficam em `docs/evidence/sprint06/`. Não usar um executável de desenvolvimento como substituto do instalador.

## Requisitos de execução

Windows11 x64, perfil Windows próprio, Microsoft Edge WebView2 Runtime e acesso HTTPS aos serviços Google. A aplicação não exige Node, npm, Rust, MSVC, Vite nem servidor próprio para uso; a prova efetiva em Windows sem essas ferramentas continua pendente até haver ambiente separado. O instalador usa o bootstrapper oficial para WebView2 quando necessário; falta de rede/permissão pode impedir essa etapa.

## Instalar

1. Obter do mantenedor o candidato autorizado e seu SHA-256, sem baixar cópias de terceiros. Conferir `Get-FileHash -LiteralPath '<caminho do instalador>' -Algorithm SHA256` contra o manifesto da compilação em `docs/evidence/release-integration/artifact-manifest.json`.
2. Fechar voluntariamente o Sheets Monitor antes de atualizar uma instalação. Preservar o perfil; não apagar SQLite/cofre para contornar erro de migração.
3. Abrir o instalador NSIS e revisar a pasta de instalação por usuário e a licença. Um candidato sem assinatura terá reputação Windows não comprovada; não contornar um bloqueio de política. Assinatura e publicação são decisões do mantenedor.
4. Abrir o aplicativo somente no ambiente de homologação autorizado. Nesta sprint a prova automatizada local de instalação não inicia o produto no perfil real.
5. Seguir [primeiro acesso](first-access.md). Sem cliente OAuth provisionado, pedir ao mantenedor um build preparado; o usuário final não deve criar projeto Cloud nem compartilhar credenciais.

A pasta alternativa de instalação compartilha identidade/cofre/dados com o produto. Não é uma sandbox. Não executar duas instalações para isolar contas. A instância única restaura a janela existente.

## Remover e preservar dados

Usar Aplicativos instalados do Windows ou o desinstalador da instalação correspondente. Não remover outra instalação. Dados de perfil e credenciais são separados dos binários; desinstalar não equivale a revogar a concessão Google. Para encerrar a sessão, desconectar pelo aplicativo; para revogar, usar a ação explícita separada antes de remover, se desejado. Não prometer limpeza de dados que não foi comprovada.

O banco operacional fica em `%APPDATA%\com.sheetsmonitor.desktop\foundation-v1.db`; contém metadados e referências privadas. Tokens ficam no cofre Windows. Nunca anexar esses dados a relatórios públicos. Migrações aplicadas são preservadas; um banco de versão futura é recusado em vez de rebaixado.

## Reproduzir o build (mantenedor)

Na raiz do repositório, com Node/npm do lockfile, Rust>=1.90 e MSVC/Windows SDK:

```powershell
npm ci
npm run lint
npm run format:check
npm run typecheck
npm test -- --maxWorkers=1
npm run build
.\scripts\check-rust.ps1 -Task format-check
.\scripts\check-rust.ps1 -Task test
.\scripts\check-rust.ps1 -Task clippy
node scripts/verify-repository.mjs --evidence-directory docs/evidence/sprint06/repository-reproduction
. ./scripts/environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis
```

Os avisos de terceiros usados no pacote estão em `distribution/`; o processo de geração consta de [instalação](installation.md). O comando Tauri compila com assets incorporados e produz NSIS x64. Registrar hash e estado OAuth/assinatura sem valores privados em um novo diretório de evidências. Configuração de OAuth é descrita em [OAuth de distribuição](distribution-oauth.md). Publicação foi autorizada, mas depende dos critérios comprovados. Reexecuções de evidência devem usar diretórios novos conforme parâmetros dos scripts.
