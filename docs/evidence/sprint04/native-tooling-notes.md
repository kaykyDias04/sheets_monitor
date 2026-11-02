# Observações de ferramentas na validação nativa

O CLI agent-browser retornou EOF no comando de viewport; leituras independentes confirmaram 780 e 1280 pixels, sem overflow horizontal. A resposta do comando não foi tratada como prova de aplicação do efeito.

Uma instância diagnóstica não encerrou o processo após pedidos normais e bloqueou a substituição do executável no Windows. A conexão CDP foi encerrada; somente o PID de teste, com caminho verificado e sem janela, foi finalizado. O erro de build foi preservado em rust-build-windows-lock-failure.log e o rebuild passou. A instância diagnóstica final encerrou normalmente após desconectar o diagnóstico; porta 9234 confirmada fechada.

No probe sem diagnóstico, WaitForInputIdle lançou uma exceção de inicialização; não foi usado como evidência de prontidão. Após breve espera, CloseMainWindow retornou true e WaitForExit confirmou término. normal-window-close.json registra essas verificações independentes. A janela foi reaberta sem argumentos de diagnóstico. Nenhum outro processo foi encerrado.

Capturas são do renderer WebView2. A imagem pública teve rótulos de conta/arquivos e inicial do avatar substituídos temporariamente no DOM, sem alterações no SQLite. HTML bruto do Axe e backups privados ficam fora do Git. As evidências públicas de acessibilidade contêm somente contagens e identificadores de regras.

O bloqueio Windows persistente do artefato compilado anterior foi resolvido preservando-o em .tooling, com origem/destino absolutos verificados dentro do workspace. Fontes, bancos e instalações existentes não foram movidos. As tentativas falhas estão preservadas em history-default/rust-test-*-windows-lock.log; o rerun passou em188 unitários e2 integrações. A instância final pós-ajuste foi encerrada normalmente após desconexão do diagnóstico, sem rascunho de regras presente. A preferência de histórico de teste foi restaurada90antes do fechamento.

Um primeiro probe de política usou um seletor que também contava a navegação interna dos detalhes. O probe foi corrigido para .sidebar nav e os valores/efeitos foram verificados independentemente antes da repetição; o resultado positivo está em native-history-default.json. A lista de cadastros mudou durante a validação e foi preservada; os logs representam estados observados em momentos diferentes.
