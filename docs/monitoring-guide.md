# Cadastro e monitoramento

O registro admite30 arquivos presentes, incluindo pausados. A descoberta Drive é paginada e não cadastra todos os resultados automaticamente. URL continua disponível quando a descoberta estiver desligada ou incompleta.

## Configurar uma proteção

Cadastre somente arquivos cuja leitura esteja autorizada. Selecione todas as abas desejadas ou um subconjunto. Defina regiões, colunas/chaves, fórmulas críticas e dependências de script conforme o escopo. A cobertura é a das regras e regiões aprovadas; uma amostra não comprova a planilha inteira.

A aplicação pode propor regras, mas o usuário deve revisar e salvar. Capture uma prévia completa e aprove visualmente a referência somente após conferi-la. Alterar abas/regras exige baseline correspondente e pode retornar Configuração pendente. Versões antigas permanecem imutáveis. Consulte [regras](integrity-rules-ui.md) e [aprovação visual](baseline-ui.md).

## Operar

Na visão geral, Iniciar monitoramento ativa o serviço Rust para arquivos elegíveis. O intervalo inicial é60s, configurável até300s, com no máximo três verificações globais e uma por arquivo. Ações manuais compartilham a fila. Uma condição de capacidade insuficiente deve ser investigada; não representa garantia de frequência cumprida em toda rede/tamanho de planilha.

Online significa obrigações aprovadas satisfeitas no escopo observado. Offline indica violação crítica comprovada. Indeterminado registra falta de prova, falha de rede/permissão/API ou ambiguidade; não é convertido em Offline por conveniência. Verificando indica trabalho em andamento. Mudanças permitidas de conteúdo geram histórico sem necessariamente danificar integridade.

Pause um arquivo para suspender sua verificação ou pause o monitoramento geral. Editar/remover cadastro são operações locais; não excluem planilhas Google. Histórico padrão é90dias, ajustável conforme a proteção; retenção preserva referências e estado necessário à deduplicação.

## Segundo plano e alertas

Fechar com X oculta para bandeja quando disponível. Sem bandeja utilizável, a janela permanece acessível. Sair encerra voluntariamente os serviços. Não matar o processo para simular saída limpa. Autostart é opt-in e não foi habilitado pelo agente.

O histórico mantém eventos e estado das tentativas de alerta. Submissão nativa registrada significa chamada aceita, sem provar exibição ou leitura. Windows/Não incomodar podem bloquear. Preferência desligada suprime eventos sem backfill ao ativar; reenvio de bloqueados é ação explícita e respeita retenção. Não há promessa de exatamente uma exibição após crash.

O produto somente lê Sheets, Drive e Apps Script, sem executar scripts. Login, revogação, aprovação de referência e alertas reais não são substituídos por fixtures.
