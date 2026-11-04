# Instalação Windows x64

Estado da homologação: consultar `SPRINT_06_PROGRESS.md`. Este guia não comprova que um instalador foi gerado ou aprovado.

Requisitos: Windows 11 x64 e Microsoft Edge WebView2 Runtime. O pacote é por usuário; eventual instalação do WebView2 pode exigir conexão/permissão própria. Node, Rust e Visual Studio são ferramentas de desenvolvimento e não são requisitos do aplicativo instalado.

1. Obter o instalador NSIS autorizado e comparar seu SHA256 com o manifesto de artefatos da mesma compilação. Não usar o executável diagnóstico `native_probe`.
2. Executar o instalador no perfil Windows desejado. O nome/versão do arquivo são os efetivamente registrados no manifesto, sem presumir uma versão diferente.
3. Abrir Sheets Monitor pelo atalho instalado, com os servidores de desenvolvimento desligados. Confirmar abertura, navegação e persistência ao sair/reabrir.
4. Conectar a conta pelo navegador do sistema, quando a configuração OAuth de distribuição estiver disponível. Nunca fornecer senhas, tokens ou credenciais Cloud no chat. A publicação externa Google exige projeto/consentimento apropriados; um instalador local não comprova essa publicação.
5. Selecionar recursos de leitura, revisar regras e aprovar pessoalmente a referência desejada. Cadastros sem referência compatível permanecem pendentes. O agente não aprova baselines reais.

O fechamento da janela oculta para a bandeja quando disponível. Para encerrar, usar **Sair** no menu da bandeja. Autostart é opcional e começa desligado. Notificações dependem das preferências do aplicativo e das permissões do Windows; aceitação da API não comprova exibição ou leitura.

Desinstalar por **Configurações → Aplicativos → Aplicativos instalados → Sheets Monitor**. Encerrar pela bandeja primeiro. Não presumir exclusão de dados locais ou revogação Google: logout local e revogação são operações distintas. Preservar bancos/configurações reais durante homologação; não remover dados do usuário para simular uma instalação limpa.

Tokens são mantidos pelo backend/cofre Windows, fora de SQLite e dos relatórios. Metadados, referências, eventos e preferências locais podem conter informação privada e dependem da proteção do perfil/disco. Não compartilhar bancos reais como evidência. O acesso Google da V1.0 é somente leitura; nenhuma etapa de instalação autoriza editar arquivos Google.

Os avisos de terceiros acompanham o pacote em `THIRD-PARTY-NOTICES.txt`. O inventário inclui conservadoramente dependências de build/teste e não afirma que todas estejam no executável. Links para as fontes das versões Rust estão junto aos avisos. Não foram alteradas fontes de terceiros. O aplicativo não redistribui SDKs/toolchains nem um runtime WebView2 próprio.

Build local autorizado, após os gates:

```powershell
. ./scripts/environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --offline --filter-platform x86_64-pc-windows-msvc --format-version 1 > .tooling/cargo-metadata.json
node scripts/distribution-notices.mjs
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis
```

O comando de avisos deve terminar sem lacunas antes de distribuir. A saída padrão esperada fica em `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`; confirmar o caminho real e o hash no manifesto. Não publicar releases nem distribuir a terceiros sem autorização específica.
