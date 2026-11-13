# Cadastro e monitoramento

O registro admite30 arquivos presentes, incluindo pausados. A descoberta Drive é paginada e não cadastra todos os resultados automaticamente. URL continua disponível quando a descoberta estiver desligada ou incompleta.

## Configurar uma proteção

Cadastre somente arquivos cuja leitura esteja autorizada. Selecione todas as abas desejadas ou um subconjunto. Defina regiões, colunas/chaves, fórmulas críticas e dependências de script conforme o escopo. A cobertura é a das regras e regiões aprovadas; uma amostra não comprova a planilha inteira.

A aplicação pode propor regras, mas o usuário deve revisar e salvar. Capture uma prévia completa e aprove visualmente a referência somente após conferi-la. Alterar abas/regras exige baseline correspondente e pode retornar Configuração pendente. Versões antigas permanecem imutáveis. Consulte [regras](integrity-rules-ui.md) e [aprovação visual](baseline-ui.md).

## Operar

Na visão geral, Iniciar monitoramento ativa o serviço Rust para arquivos elegíveis. O intervalo inicial é 60s, configurável entre 60 e 600s, com no máximo três verificações globais e uma por arquivo. Ações manuais compartilham a fila. O próximo prazo considera a conclusão, sem sobrepor nem reproduzir ciclos perdidos. Uma condição de capacidade insuficiente deve ser investigada; não representa garantia de frequência cumprida em toda rede/tamanho de planilha.

Online significa obrigações aprovadas satisfeitas no escopo observado. Offline indica violação crítica comprovada. Indeterminado registra falta de prova, falha de rede/permissão/API ou ambiguidade; não é convertido em Offline por conveniência. Verificando indica trabalho em andamento. A linha do tempo e seus exports mostram somente atualizações de regras; edições livres não são alertas de proteção. Diagnósticos de consulta permanecem em Monitoramento.

Pause um arquivo para suspender sua verificação ou pause o monitoramento geral. Editar/remover cadastro são operações locais; não excluem planilhas Google. Histórico padrão é90dias, ajustável conforme a proteção; retenção preserva referências e estado necessário à deduplicação.

## Segundo plano e alertas

Fechar com X oculta para bandeja quando disponível. Sem bandeja utilizável, a janela permanece acessível. Sair encerra voluntariamente os serviços. Não matar o processo para simular saída limpa. Autostart é opt-in e não foi habilitado pelo agente.

O histórico mantém eventos e estado das tentativas de alerta. Submissão nativa registrada significa chamada aceita, sem provar exibição ou leitura. Windows/Não incomodar podem bloquear. Preferência desligada suprime eventos sem backfill ao ativar; reenvio de bloqueados é ação explícita e respeita retenção. Não há promessa de exatamente uma exibição após crash.

O produto somente lê planilhas e Apps Script, sem executar scripts. Com sincronização opcional habilitada, grava apenas configurações na pasta privada do app no Drive. Login, revogação, aprovação de referência e alertas reais não são substituídos por fixtures.

## Exportar atualizações

Em Atividades, selecione período, planilha, aba e tipo de evento e use Exportar logs. CSV UTF-8 com BOM, XLSX nativo e JSON estão disponíveis. O export aplica todos os filtros ao conjunto completo, não apenas à página atual, em ordem cronológica. Datas da interface abrangem o dia local inteiro, convertido para UTC na consulta; nenhuma correspondência cria arquivo vazio. CSV neutraliza células que poderiam ser interpretadas como fórmulas; XLSX grava texto literal, inclusive conteúdo iniciado por `=` ou URLs. O XLSX admite até 1.048.575 registros mais o cabeçalho e 32.767 unidades UTF-16 por célula; ao exceder esses limites, a exportação é recusada sem truncar dados. Restrinja os filtros ou use CSV/JSON. A escrita XLSX usa armazenamento temporário e exige espaço e permissão nas pastas temporária e de destino.

Falhas de autenticação, rede, permissão, sincronização e cota trazem causa, impacto e solução. Aguarde o prazo de cota antes de repetir consultas; aumentar o intervalo pode reduzir pressão, mas não corrige permissões. Nenhuma mensagem autoriza correção automática ou garante entrega visual do Windows.


## Recuperar configurações em outro computador

Em Configurações, marque Sincronizar configurações entre dispositivos e salve/autorize no navegador. Repita no outro computador com a mesma conta Google e aplicação OAuth. Um perfil vazio recupera cadastros, abas, regras e referências aprovadas automaticamente; preferências locais de inicialização não são transferidas. Sem rede, os dados locais permanecem disponíveis e serão reconciliados após a recuperação.

Se houver configurações divergentes, escolha explicitamente manter a local ou usar uma versão remota. Nada é substituído automaticamente nesse caso; cópias e referências anteriores ficam preservadas no SQLite. Se outra alteração ocorrer antes da decisão, atualize o conflito e decida novamente. A nuvem mantém commits imutáveis; atingir o limite de listagem interrompe a sincronização com erro, sem truncar dados. Após restauração, a primeira leitura estabelece uma nova observação local e confirma o acesso aos recursos; referências restauradas não concedem permissões Google.

Para consultar editores do arquivo, habilite Auditoria Workspace e autorize com uma conta com privilégio de auditoria. No diagnóstico de um evento, use Consultar auditoria Workspace. Esses editores não são presumidos responsáveis pela diferença detectada. A ausência de registros pode indicar atraso, retenção, permissão ou identidade indisponível.
