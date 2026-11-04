# Gate 1 — prontidão e caminho de evidência

Revisor: lane oracle independente da implementação. Tentativa 1 de 3, em 03/11/2026. Base inspecionada: b2c68ce e árvore documental local preservada. Resultado: **APROVADO para iniciar a fase 2**, condicionado à conclusão sem falhas do baseline já em execução. Este gate aprova o plano e o isolamento; não aprova a V1.0 nem converte testes em andamento em testes concluídos.

## Material examinado

Prompt original em `docs/sprint06.original.md`, estado `.slim/deepwork/sprint06-homologation.md`, proposta `integration-load-plan.md`, verificador do repositório, registro das migrações, admissão `monitoring/work.rs`, ciclo do serviço `monitoring/service.rs`, `notifications/service.rs`, fronteira de aprovação de referência em `database/integrity_store.rs`. Readiness/security finais ainda não estavam disponíveis na revisão inicial; seus achados materiais devem ser reconciliados antes do gate 2.

## Decisão e limites

- A campanha proposta atravessa o coordenador real, leases, avaliação, persistência em disco e outbox. O sink deve ser estritamente simulado e as credenciais/IDs inteiramente sintéticos. Nenhum comando IPC ou feature de produção novo é necessário.
- Contadores e asserções devem comprovar máximo global três, exclusão por arquivo, cardinalidade dos eventos e ausência de novas entregas na repetição. Não basta produzir métricas sem conferir invariantes.
- Relógio lógico acelerado deve ser separado do tempo de parede e do relógio UTC do dispatcher. Datas incompatíveis com TTL não podem gerar aprovação artificial. Ensaios 5/15/30 precisam de resultados separados e execução efetiva; CPU/memória devem identificar PID e janela medida.
- Não transformar 24 horas lógicas em 24 horas reais. Sem recursos/referências humanas, campanha Google continua não executada; sem requisitos essenciais comprovados a classificação final é BLOQUEADA, conforme o Prompt 06.
- Preservar os cinco arquivos de migração e evidências anteriores. O verificador pode reconhecer a autorização do Prompt 06 e ganhar diretório de saída próprio; não remover os requisitos de evidência de conclusão nem regravar JSON histórico da Sprint 05.

## Controles indispensáveis para a fase 3

Diretório alternativo de instalação **não isola** a identidade Tauri, o cofre, app_data_dir, a instância única nem as chaves de desinstalação NSIS. Antes de instalar, conferir colisões da identidade em HKCU/HKLM e atalhos; preservar qualquer instalação já existente. Não iniciar o produto instalado no perfil atual com dados/cofre reais. Homologação de execução instalada exige VM ou usuário Windows isolado. Instalação/desinstalação de um pacote novo, quando não houver colisão, pode produzir evidência limitada de empacotamento, sem provar execução limpa, autenticação ou notificações instaladas.

Versão 1.0.0 de um artefato local candidato não equivale a release aprovada. Sem assinatura, provisionamento OAuth ou ambiente limpo, registrar os respectivos limites e preparar checkpoint concreto. Não publicar nem distribuir a terceiros. Notificação real e autostart permanecem ações humanas separadas.

## Próximo gate

Gate 2 deve confrontar resultados efetivos, falhas preservadas, código final de testes, métricas por tamanho, riscos da auditoria e ausência de enfraquecimento das regras. Revisão de fatos históricos não substitui revalidação do artefato final.

## Recurso externo confirmado pelo usuário

Durante o gate, o usuário confirmou ausência de ambiente Windows separado e das 30 planilhas com referências aprovadas. A fase 2 pode executar testes locais sintéticos e a fase 3 pode preparar o candidato, respeitando a preflight de instalação. Campanha Google de 30, execução instalada em ambiente limpo e demais critérios essenciais sem prova permanecem pendentes; o parecer final deve ser **BLOQUEADA**, e não PENDENTE DE HOMOLOGAÇÃO apenas aguardando assinatura humana. Não iniciar o produto instalado no perfil atual.
