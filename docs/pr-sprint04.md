# PR local — Sprint 04: interface e gerenciamento

A navegação diária ainda dependia de dados demonstrativos e operações técnicas. Esta mudança entrega dashboard da conta conectada, descoberta Drive paginada, cadastro por link, seleção de abas, editor de regras, revisão/aprovação de baselines e histórico/diagnóstico consultados no backend. Cadastro novo aparece pendente; não existe Online padrão nem atualização automática de referência.

SM-018 a SM-021 e adições autorizadas SM-021A/SM-021B. Migração 0004 acrescenta registro operacional e preferências, preservando as três anteriores. O limite de 30 arquivos é transacional, incluindo pausados e considerando as contas locais; abas têm limite independente e orçamento total de células. A descoberta pode listar mais de 30 arquivos e informa incompletude.

O cliente Desktop da aplicação pode ser provisionado no build pelo mantenedor; uso diário não exige credenciais Cloud. Permissões já existentes de metadados Drive continuam opt-in e verificadas. Não há servidor próprio, escrita Google, execução de scripts, scheduler, notificações nativas, bandeja, autostart ou instalador.

Revisões corrigiram concorrência de sessão, aprovação/revisão e commit de avaliações; indicação de referência ativa entre escopos; seleção de mais de 30 abas; abertura de URL segura no navegador. A remoção é local e oferece preservar ou excluir atividades, mantendo baselines imutáveis em ambos os casos com aviso explícito.

Evidências de testes, build e homologação delimitada serão consolidadas no encerramento em docs/evidence/sprint04. Sucessos de fixtures permanecem separados das leituras reais. Não se declara publicação Google externa nem CI hospedado. Este arquivo é apenas o rascunho local; nenhum PR foi publicado sem remoto autorizado.
