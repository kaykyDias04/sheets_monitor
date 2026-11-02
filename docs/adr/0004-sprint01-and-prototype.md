# ADR 0004 — Sprint 01 e preservação do protótipo

Estado: aceito após o usuário substituir o Prompt 02 pela Sprint 01 do anexo v1.0.

O protótipo anterior continha OAuth, APIs e monitoramento implementados antes do contrato mestre. Estava fora do histórico Git e não tinha homologação nativa. A solicitação atual exige uma fundação sem implementar ainda autenticação Google ou monitoramento real.

Preservamos 47 arquivos no commit `ffcaa49`, copiamos seus bytes para `archive/prototype/` e registramos hashes. O arquivo é referência histórica, fora do build ativo. Não executar, importar ou formatar esse material. Componentes ativos podem ser substituídos pela fundação somente após preservar seu original.

O anexo original é mantido em `docs/specification-v1.0.original.md`. `docs/requirements.md` é sua leitura operacional e os IDs SM permanecem exatamente os do anexo. Datas/estimativas são planejamento, não promessa nem autorização. O modelo de Prompt 01 do anexo foi explicitamente autorizado pelo usuário em resposta à pergunta de escopo.

A aplicação ativa usa apenas fixtures e persistência local, com aviso tanto no navegador quanto no desktop. Módulos futuros são contratos sem implementação. O banco `foundation-v1.db` é separado dos bancos anteriores. Não migrar dados reais nem apagar arquivos antigos para fazer a demonstração funcionar.

Os parâmetros anteriores do protótipo (intervalo até 24h, integridade binária, ausência de seleção de abas e análise de script apenas por indicadores) não definem a V1 aprovada. Devem ser substituídos, nas sprints autorizadas, por 60–300s, quatro estados, escopo por abas e validação estática de referências.

Consequência: a fundação pode reaproveitar ideias e ativos próprios, mas não marcar SM-007 a SM-028 como concluídos pela existência de arquivos arquivados. Ao concluir Sprint 01, aguardar nova instrução.
