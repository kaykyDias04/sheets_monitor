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
