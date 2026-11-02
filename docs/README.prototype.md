# Sheets Monitor

Aplicativo local para acompanhar até 30 planilhas Google e scripts Apps Script autorizados. React + TypeScript, Tauri 2, Rust e SQLite. Interface em português.

O diretório inicial estava vazio. [Requisitos provisórios](docs/requirements.md) registram o contrato recebido; o `requirements.md` original citado não foi fornecido.

## Executar

Windows 11, Node.js e Rust estável. Para o build padrão, instale Microsoft C++ Build Tools com Desktop development with C++ e Windows SDK; WebView2 deve estar disponível. Consulte os [pré-requisitos oficiais Tauri](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib
npm run tauri dev
```

`npm run dev` abre somente a prévia web com dados sintéticos em memória. A integração Google e a persistência SQLite pertencem ao aplicativo desktop. A prévia não representa uma sessão real do Windows/Google.

```powershell
npm run tauri build
```

O empacotamento NSIS gera instalador em `src-tauri/target/release/bundle/nsis/`. A versão local não é assinada. Nenhum servidor hospedado é necessário.

## Conectar Google

1. No seu projeto Google Cloud, habilite Sheets API, Drive API e Apps Script API. Configure a tela de consentimento e o usuário de teste, quando aplicável.
2. Crie um cliente OAuth do tipo **Aplicativo para computador**. Não use cliente web nem service account.
3. Informe o client ID na interface desktop e use Conectar. Se o cliente exigir o client secret desktop, configure `SHEETS_MONITOR_CLIENT_SECRET` no ambiente do processo antes de iniciar. Não coloque credenciais no código, no histórico Git ou em capturas.
4. O navegador do sistema solicitará os três escopos de leitura. A aplicação recebe o retorno em `127.0.0.1` com porta efêmera e valida state e PKCE. Tokens ficam no Windows Credential Manager.
5. Selecione o modo Google, cadastre somente os IDs das planilhas e dos scripts autorizados. Revise a prévia e aprove cada baseline explicitamente.

O aplicativo não escreve nas planilhas, não executa scripts e não publica projetos. Desconectar remove a credencial local; a concessão remota pode ser administrada na sua conta Google. Sem credenciais e consentimento, a integração real permanece **não validada**.

## Monitoramento

Cada campo diferente nos snapshots completos gera um evento com antes/depois e uma entrada persistente na fila de notificações. O histórico permite inspecionar todas as diferenças detectadas, inclusive fontes e manifestos de scripts. A análise de scripts é heurística e não executa o código.

Baselines e históricos de demonstração ficam separados dos reais. O agendador funciona enquanto o processo está aberto. Uma coleta com erro não substitui o snapshot anterior. Notificações podem ser repetidas após interrupção, seguindo entrega pelo menos uma vez. Aceitação pelo sistema não garante que o Windows exibiu o toast.

O polling não vê mudanças transitórias revertidas entre coletas. Só observa os campos disponibilizados pelas APIs e pelos acessos concedidos; não descobre automaticamente projetos Apps Script vinculados. Respostas maiores que 64 MiB e paginação acima do limite de proteção geram erro explícito, sem sucesso parcial.

SQLite e relatórios contêm conteúdo das planilhas e fontes autorizadas em texto local; proteja a conta Windows e o disco. Tokens não entram nesses arquivos. Use dados sintéticos ao compartilhar evidências.

Veja [plano de verificação](docs/verification.md), [decisões](docs/adr/0001-local-first-and-boundaries.md), [núcleo](docs/core.md), [integração Google](docs/google-integration.md), [interface](docs/ui.md) e [evidências](docs/evidence/).
