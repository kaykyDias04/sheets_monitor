# Verificação nativa — Sprint 01 aprovada tecnicamente

O executável standalone MSVC x64 final (hash em native-artifact.json) abriu uma janela real do Sheets Monitor no Windows 11. O lançamento final normal, sem flags de diagnóstico, está em native-normal-launch.json; Vite não estava em execução e a porta diagnóstica foi encerrada.

## Método e limitação de captura

O helper Windows retornou uma captura que não correspondia à janela selecionada. A imagem alheia foi descartada, não foi salva no repositório e nenhuma coordenada foi usada. A inspeção foi realizada no próprio WebView2 do executável por CDP, em loopback e com perfil isolado, apenas no ambiente do processo. Não se alterou registro nem variáveis persistentes de usuário/máquina.

A página inspecionada era `http://tauri.localhost`, com `__TAURI_INTERNALS__` presente e assets incorporados correspondentes ao build final. Isso diferencia a aplicação nativa da prévia Vite. As capturas mostram **o conteúdo do renderer da instância nativa, sem a moldura do Windows**. O título e o handle da janela foram verificados por metadados do processo.

## Cenários efetivamente exercidos

1. Dashboard: quatro exemplos, quatro estados e aviso de demonstração.
2. Planilhas: listagem, filtro/busca disponíveis e cadastro Google indisponível.
3. Histórico: eventos sintéticos identificados, com categorias e diagnóstico.
4. Configurações: persistência SQLite e versão do banco visíveis; Google/autostart indisponíveis.
5. Preferência alterada pela UI de60 para120, salva pelo comando Tauri e conferida por conexão SQLite readonly.
6. Processo fechado e reaberto; UI e SQLite recuperaram120, mantendo quatro arquivos e um ledger de migração. Onze tabelas de produto presentes.
7. Padrão60 restaurado pela interface; nenhuma baseline foi aprovada.
8. Instância de diagnóstico encerrada e aplicação reaberta normalmente. Porta9227 fechada e porta1420 sem listener.

O encerramento da instância de QA foi controlado com Stop-Process após salvar os dados; a saída -1 do host corresponde a esse encerramento, não é relatada como crash da aplicação. A instância normal permaneceu aberta para inspeção do usuário.

## Acessibilidade

O primeiro audit encontrou contraste insuficiente. O relatório original e as capturas before-contrast foram preservados. A paleta foi corrigida sem mudar o layout; oito testes frontend e build foram repetidos, seguidos de rebuild nativo com os novos assets.

Axe4.12.1 no renderer final retornou **zero violações e zero itens incompletos nas quatro telas**. Os relatórios individuais native-*-a11y.json registram os resultados. Isso é evidência automatizada delimitada, não certificação completa WCAG.

## Evidência e escopo

native-qa.json consolida navegação, persistência, audits e limites. native-sqlite-*.json registra apenas contagens e preferências sintéticas, sem conteúdo privado. As imagens native-dashboard.png, native-spreadsheets.png, native-history.png e native-settings.png mostram o renderer final; native-settings-after-restart.png mostra120 após reabertura.

Não houve login Google, consulta de arquivos reais, aprovação humana simulada, execução de Apps Script, scheduler, autostart, bandeja ou instalador. CI GitHub não executado porque não há remoto autorizado.

Método de diagnóstico documentado pela Microsoft: [WebView2 globals](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl) e [WebView2/CDP](https://playwright.dev/docs/webview2).
