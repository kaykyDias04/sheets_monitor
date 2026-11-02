# Sheets Monitor — contrato mestre de execução

## Autoridade e escopo

O Prompt 00 estabelece exclusivamente as regras permanentes de comportamento, autonomia, segurança e qualidade. Ele substitui a autorização anterior de implementação integral e de avanço automático entre fases.

**Não iniciar nem continuar a implementação do aplicativo no Prompt 00.** A implementação será autorizada exclusivamente por prompts posteriores, numerados de 01 a 15. Conhecer o escopo completo não autoriza implementá-lo.

Cada prompt numerado autoriza somente sua própria etapa. Não implementar funcionalidades de etapas posteriores, não antecipar tarefas por dependências futuras e não avançar automaticamente para o próximo prompt. Interfaces, contratos e estruturas podem ser preparados somente quando necessários à etapa atual, sem implementar módulos futuros.

## Autonomia durante a etapa autorizada

Realizar autonomamente as operações técnicas possíveis dentro do escopo: criar, modificar e organizar arquivos; instalar e configurar dependências permitidas; executar comandos e scripts; escrever e refatorar código; executar testes; identificar e corrigir erros; gerar documentação; realizar commits em branches de trabalho; preparar builds e instaladores quando autorizados.

Não solicitar confirmação para decisões técnicas rotineiras e reversíveis dentro da etapa autorizada.

## Encerramento obrigatório de cada etapa

1. Executar os testes pertinentes.
2. Corrigir automaticamente os erros identificados.
3. Registrar alterações e evidências verificáveis.
4. Atualizar o relatório de progresso.
5. Informar funcionalidades concluídas, testes realizados e pendências.
6. Identificar qual deverá ser o próximo prompt.
7. **Encerrar a execução e aguardar o envio do próximo prompt numerado.**

Não considerar funcionalidades concluídas sem evidências. Manter documentação, testes automatizados e registros de decisões arquiteturais. Não afirmar execução de testes sem evidência.

## Segurança e intervenção humana

Minimizar a participação humana; solicitar intervenção somente quando indispensável, especialmente para autenticação, consentimento OAuth, credenciais indisponíveis, privilégios administrativos, aprovação de baseline ou operações externas sensíveis.

Nunca contornar permissões nem expor segredos. Não alterar arquivos reais do Google sem aprovação.

## Projeto de referência

Sheets Monitor: aplicativo desktop para Windows 11, Tauri 2, React, TypeScript, Rust e SQLite, integrado às APIs Google Sheets, Drive e Apps Script. Esta descrição não constitui autorização para implementar qualquer etapa.

## Estado no Prompt 00

Contrato registrado. Trabalho iniciado sob a solicitação anterior foi interrompido; os arquivos existentes foram preservados. Não instalar dependências, inicializar projeto ou implementar funcionalidades nesta etapa. Nenhuma conclusão de integração ou testes é inferida deste registro.

**Próxima ação permitida: receber o Prompt 01 e executar apenas o escopo que ele autorizar.**

## Atualização de escopo — Sprint 01 do anexo v1.0

O registro acima descreve o estado histórico do Prompt 00. O Prompt 01 de ambiente foi concluído. Durante o Prompt 02, o usuário forneceu `docs/specification-v1.0.original.md` e confirmou expressamente: **substituir a etapa atual pela Sprint 01 do anexo**.

A autorização vigente é criar a fundação Tauri 2 + React/TypeScript/Vite + Tailwind/shadcn, backend Rust modular, SQLite com migrações das onze entidades, navegação com dados simulados, testes, lint, formatação, CI Windows e documentação. Comprovar aplicativo em execução nativa. Não implementar autenticação Google, consultas reais ou motor de monitoramento nesta sprint.

Os IDs oficiais são os SM-001 a SM-028 do anexo; não renumerar. Subtarefas usam `SM-002.01`. A numeração dos prompts e das sprints não é intercambiável. Ao concluir esta Sprint 01, encerrar e aguardar nova instrução; não iniciar Sprint 02 por conta própria.

O protótipo anterior está preservado no commit `ffcaa49` e em `archive/prototype/`. Esse diretório é somente referência: não importar, compilar, formatar nem apresentar como funcionalidade validada. Fontes futuras já existentes ali não constituem autorização nem integração real concluída.

Documentos canônicos: `docs/requirements.md`, `docs/architecture.md`, `docs/backlog.json`, `docs/acceptance.md` e `docs/progress.md`. Documentos de protótipo e ADRs históricos podem divergir: prevalecem o contrato do usuário e o anexo v1.0, com decisões atuais registradas em novos ADRs. Mudanças de requisitos precisam de registro, não de suposição silenciosa.

Convenções e publicação: seguir `CONTRIBUTING.md`; commits em branch `codex/`, ligados a IDs SM. Sem remoto GitHub autorizado, trabalhar localmente. Não enviar mensagens, criar recursos remotos, consentir OAuth ou aprovar baseline real automaticamente.

## Autorização vigente — Sprint 02

O usuário autorizou expressamente `docs/sprint02.original.md` em 02/11/2026. A Sprint 01 foi encerrada no commit `2a0b0f8`; suas evidências são históricas e não devem ser sobrescritas por novos testes.

Implementar somente SM-007 a SM-011: OAuth desktop PKCE, identidade da conta, cofre Windows e ciclo de sessão; leitores Sheets/Drive/Apps Script; persistência de configuração/IDs/metadados sem segredos; UI de conexão e consultas manuais; testes, build e documentação. O acesso Google deve ser somente leitura e as consultas reais não podem usar fixtures como sucesso.

Não implementar Sprint 03: motor de integridade, comparação definitiva de baselines, classificação automática Online/Offline, análise avançada de dependências, agendamento contínuo ou notificações definitivas. Não alterar a migração0001 já aplicada; evoluir com novas migrações.

Google Cloud só pode ser configurado em projeto identificado e autorizado. Sem acesso disponível, implementar e testar adaptadores com mocks e registrar a homologação real como pendente. Não criar projetos duplicados, configurar faturamento ou pedir senhas/tokens no chat. Agrupar credenciais, consentimento e recursos de teste em um checkpoint quando a aplicação estiver pronta para revisão.

Login/consentimento serão feitos pelo usuário no navegador do sistema. Revogar autorização é ação explícita separada de logout local. Mudança de escopos usa nova autorização completa: Google não suporta autorização incremental em apps instalados. `drive.file` só é adequado quando a seleção por arquivo e suas permissões de escrita forem compatíveis; esta sprint não pode solicitar edição/exclusão.

Evidências atuais: `docs/evidence/sprint02/`. Atualizar backlog sem marcar integração real concluída apenas por mocks. Ao encerrar, aguardar Prompt03 sem executar a Sprint03.

## Encerramento da Sprint 02

Homologação real concluída com conta e recursos autorizados pelo usuário: login, cofre/recuperação, refresh e leituras Sheets/Drive/Apps Script. Evidências sanitizadas em docs/evidence/sprint02/google-live.json. SM-007 a SM-011 concluídos. Aguardar Prompt 03; não iniciar funcionalidades posteriores automaticamente. Credenciais e conteúdos reais continuam fora do Git e dos relatórios.
