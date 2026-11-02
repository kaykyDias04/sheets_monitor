# Desktop — Sprint 02

A entrega atual adiciona Integração Google à fundação. Somente os dois comandos foundation retornam fixtures is_demo=true. Os sete comandos integration usam cofre/backend reais, sem fallback simulado. Credenciais e consentimento continuam necessários para homologação externa. O texto abaixo registra também limites e evidências históricas da fundação.

# Fundação desktop — Sprint 01

O executável desta sprint contém Tauri 2, a interface React e um banco SQLite
local com exemplos sintéticos. Todas as planilhas e atividades expostas pelo
bridge têm `is_demo: true`. A versão `0.1.0` identifica a fundação em
desenvolvimento; não houve publicação de release nem geração de instalador.

## Executar no Windows 11

Os pré-requisitos verificados e sua configuração estão em
[environment.md](environment.md). O ambiente de compilação é Windows x64/MSVC,
com Node, Cargo, C++ Build Tools e WebView2. O script usa as funções existentes de
`scripts/environment-common.ps1`, importando VsDevCmd somente no processo atual.
Não altera o PATH global, o registro do Windows nem instalações existentes.

```powershell
npm ci
npm run build
.\scripts\check-rust.ps1 -Task test
.\scripts\check-rust.ps1 -Task clippy
.\scripts\check-rust.ps1 -Task build
.\src-tauri\target\x86_64-pc-windows-msvc\debug\sheets-monitor.exe
```

Para desenvolvimento com Vite e recarga automática, carregar o ambiente MSVC
somente no PowerShell atual:

```powershell
. .\scripts\environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
npm run tauri dev
```

Nenhum servidor hospedado
é necessário; Vite é apenas o servidor local de desenvolvimento. A compilação
desktop feita pelo helper habilita `custom-protocol` e incorpora os arquivos
estáticos de `dist/`. Um `cargo build` simples, sem essa feature, produz o modo de
desenvolvimento que usa o `devUrl`; para o executável independente use o helper.

## Fronteira nativa e persistência

O frontend invoca apenas dois comandos definidos pelo aplicativo:

| Comando | Entrada | Resultado |
|---|---|---|
| `foundation_get_dashboard` | nenhuma | planilhas, atividades, configurações e `is_demo: true` |
| `foundation_save_settings` | `{ settings }` | persiste configurações locais válidas |

O banco é criado em `app_data_dir()/foundation-v1.db`, separado dos arquivos do
protótipo. Na instalação Windows padrão, o diretório de dados corresponde a
`%APPDATA%\com.sheetsmonitor.desktop`. O schema versionado e a inserção idempotente
de demonstração são implementados no módulo `database`. Reiniciar o aplicativo
preserva as configurações sem duplicar os exemplos. Não existe comando para
apagar o diretório de dados do usuário.

O intervalo de 60–300 segundos e a retenção padrão de 90 dias são configurações
preparatórias. Nenhuma consulta periódica ou limpeza de histórico acontece nesta
sprint. O campo `start_with_windows` fica desativado: a tentativa de ativá-lo é
recusada, e nenhum mecanismo de inicialização automática é registrado.

## Módulos reservados

`auth`, `google`, `integrity`, `monitoring` e `notifications` definem somente
contratos Rust, sem implementações registradas. Esta sprint não abre navegador de
consentimento, não acessa Google, não armazena tokens, não aprova baselines, não
avalia integridade real e não envia notificações. Os estados exibidos nas telas
pertencem aos exemplos, sem representar verificações reais.

O protótipo anterior permanece em `archive/prototype/`, fora do crate ativo.
[prototype-isolation.json](evidence/sprint01/prototype-isolation.json) registra
os SHA-256 comparados antes da remoção dos sete arquivos antigos do código
ativo. O protótipo e seus relatórios históricos não comprovam a Sprint 01.

## Superfície de acesso

O backend da fundação não depende de bibliotecas OAuth, cofre de tokens ou
notificação. A capability da janela não concede shell, filesystem ou HTTP ao
frontend. A CSP limita scripts aos arquivos do aplicativo e conexões ao IPC
local. Recursos externos não são necessários para executar a interface.

O bundle está desativado na configuração para respeitar o escopo da Sprint 01.
Preparar e validar o instalador pertence a SM-028, na Sprint 06.

## Evidências

Os comandos de `check-rust.ps1` registram saída e falham com exit code não zero
quando o Cargo falha. Os logs ficam em `docs/evidence/sprint01/`; somente um log
de execução concluída permite declarar que aquele gate passou. A validação da
janela em execução e da navegação é registrada separadamente pelo agente
principal, sem confundir testes unitários com homologação visual.

A execução registrada nesta sprint passou em 12 testes unitários do crate
desktop e dois testes de integração, Clippy com `-D warnings`, rustfmt e build
MSVC x64 com assets incorporados. Comandos, códigos de saída e limites estão em
[native-gates.json](evidence/sprint01/native-gates.json). O hash do executável e
dos assets está em [native-artifact.json](evidence/sprint01/native-artifact.json).
Essas evidências não substituem o registro separado da janela real e da navegação.
