# Interface operacional — Sprint 04

A interface operacional usa dados persistidos da conta conectada. A demonstração da fundação não é carregada nas áreas de uso diário. Fora do Tauri, a interface informa a necessidade do aplicativo Windows; não simula conexão nem sucesso.

O fluxo é conectar Google, autorizar descoberta, escolher arquivos, selecionar abas, configurar regras, capturar/revisar a referência e aprová-la explicitamente. As áreas são Visão geral, Planilhas, detalhes por arquivo, Atividades, Regras e baselines e Configurações. Ferramentas técnicas permanecem em uma área avançada.

Online significa avaliação concluída pelo motor. Arquivos novos ficam em Configuração pendente; com referência e sem avaliação, Ainda não verificada. Pausada é uma condição de operação, separada de uma falha estrutural. Indeterminado significa que não foi possível determinar a integridade; conectividade não substitui esse diagnóstico.

A Sprint 04 permite verificações sob demanda. Preferências de intervalo e notificações podem ser salvas como preparação; o agendador, avisos Windows, bandeja e inicialização automática pertencem à Sprint 05. Nenhum controle deve sugerir que esses serviços já executam.

Descoberta usa metadados sem baixar células. Histórico usa busca, filtros e paginação no SQLite. Nenhuma alteração é atribuída a uma pessoa sem evidência de autoria. Remoção altera somente o cadastro local; o arquivo Google permanece intacto.

Evidências da implementação e testes ficam em docs/evidence/sprint04. Este documento descreve a experiência pretendida e não substitui os registros de execução.
