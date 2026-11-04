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

## Autorização vigente — Sprint 03

O usuário autorizou docs/sprint03.original.md. Implementar somente SM-012 a SM-017: snapshots normalizados, baselines versionados com aprovação explícita, diff estrutural/conteúdo/fórmulas, análise determinística Apps Script, classificação e eventos deduplicados, retenção e operações sob demanda. Preservar Sprints01/02 e migrações já aplicadas; evidências novas em docs/evidence/sprint03.

Permitir interface técnica simples, sem dashboard/gerenciamento/histórico definitivos, scheduler, notificações ou instalador das Sprints04–06. Não alterar arquivos Google nem aprovar baseline real automaticamente. Regras e mutações sintéticas ficam em fixtures; coletar recursos reais só em leitura autorizada, sem expor identidade, IDs, fórmulas ou fontes em relatórios públicos. Sem autorização destrutiva, registrar homologação real de mutações como pendente.

Aplicar deepwork com lanes de implementação e gates de revisão; verificar algoritmos determinísticos, ambiguidades e isolamento de conta. Ao terminar, encerrar e aguardar Prompt04.

## Encerramento técnico da Sprint 03

SM-012 a SM-017 implementados e aprovados nos gates locais. Evidências e limites em docs/evidence/sprint03/final-report.md. Baseline humana real restrita a duas células; rascunho maior não aprovado; mutações Google reais pendentes e cobertas por fixtures. Nenhuma aprovação de baseline real pelo agente. Preservar evidências históricas. Aguardar Prompt 04 antes de iniciar a Sprint 04.

## Autorização vigente — Sprint 04

O usuário autorizou docs/sprint04.original.md: SM-018 a SM-021 e adições SM-021A (conta) e SM-021B (baselines). Implementar descoberta Drive paginada, gerenciamento de até 30 planilhas, dashboard real, seleção de abas/regras, aprovação visual, histórico/diagnósticos e preferências persistidas. Preservar as sprints anteriores e registrar evidências em docs/evidence/sprint04. Não usar fixtures na interface operacional.

Usar escopos mínimos adequados; mudanças exigem consentimento oficial, sem ampliação silenciosa e sem servidor próprio. Usuário final não deve informar credenciais Cloud: configuração da aplicação pertence ao desenvolvimento/empacotamento; publicação externa depende de projeto autorizado e requisitos OAuth. Baseline real continua com aprovação humana; Google somente leitura. Aplicar deepwork com lanes e gates. Ao concluir, aguardar Prompt 05, sem scheduler, notificações nativas, bandeja, autostart ou instalador final.

## Encerramento técnico da Sprint 04

SM-018 a SM-021 e adições autorizadas SM-021A/SM-021B implementados e aprovados nos três gates. Evidências em docs/evidence/sprint04/final-report.md. Testes: 188 unitários Rust, duas integrações e 42 React aprovados; builds/checks e inspeção nativa aprovados. Descoberta e operações Google reais comprovadas no escopo registrado, sem alterações Google ou aprovação de baseline pelo agente.

A referência humana anterior cobre uma aba e duas células; a seleção atual de 28 abas está em Configuração pendente e foi preservada. Nova referência ampliada exige revisão/aprovação humana. Publicação Google externa permanece separada da validação local. Preservar migrações/evidências anteriores. Encerrar e aguardar Prompt 05, sem antecipar scheduler, notificações, bandeja, autostart ou instalador.

## Autorização vigente — Sprint 05

O usuário autorizou docs/sprint05.original.md: implementar exclusivamente SM-022 a SM-025, serviço Rust contínuo, fila e limite global de três verificações, intervalo60–300s, outbox transacional, notificações Windows, bandeja/instância única, autostart opt-in desligado por padrão e recuperação segura. Reutilizar Sprints01–04 e preservar configurações, referências e migrações aplicadas. Evidências novas em docs/evidence/sprint05 e relatório em docs/sprint-05-report.md.

Executar o agendamento sem depender do React/WebView. X oculta para bandeja quando disponível; Sair encerra voluntariamente e limpa recursos. Não modificar Google nem aprovar baselines; somente escopos válidos com referência correspondente são elegíveis. Não enviar notificações reais sem preferência/permissão apropriada; testes explícitos devem ser identificados como testes. Não confundir OS aceitou solicitação com exibição/leitura pelo usuário. Não ampliar scopes ou habilitar autostart silenciosamente.

Aplicar deepwork com lanes e gates e testes com relógio/transporte simulados. Preservar dados privados fora das evidências públicas. Solicitar intervenção apenas quando autenticação, consentimento, UAC, baseline ou confirmação humana de notificação forem indispensáveis. Não executar campanha final de carga30planilhas nem gerar instalador; isso pertence à Sprint06. Ao concluir, encerrar e aguardar Prompt06.

## Encerramento técnico da Sprint 05

SM-022 a SM-025 implementados e aprovados nos gates locais. Relatório em `docs/sprint-05-report.md`, evidências em `docs/evidence/sprint05/`. Suíte final: 257 unitários Rust, duas integrações e 50 React aprovados; um teste de cofre ignorado por padrão, implementação inalterada. Checks/build Windows aprovados. Serviço nativo oculto comprovado com recursos/relógio sintéticos; ciclo de vida e instância única comprovados no aplicativo real.

O primeiro alerta autorizado não apareceu. A falha permanece registrada; a entrega foi corrigida com WinRT e o segundo alerta, também autorizado, foi exibido e confirmado pelo usuário. O processo desse probe foi interrompido antes de completar seu relatório/callbacks; não afirmar saída limpa dessa execução específica. Provas de saída limpa sem toast são separadas. Build de desenvolvimento usa remetente Windows PowerShell explicitamente informado; release usa a identidade configurada, sem fallback após bloqueio.

A inspeção real encontrou quatro cadastros pendentes e zero escopos elegíveis; as escolhas foram preservadas e nenhuma baseline real foi aprovada pelo agente. Autostart não foi habilitado e nenhum arquivo Google foi alterado. Identidade instalada, clique físico no menu/toast, login autostart e campanha final em escala continuam com os limites documentados. Migrações/evidências anteriores preservadas. Sem remoto autorizado, commits e rascunho PR locais. Encerrar e aguardar Prompt06; não iniciar a Sprint06 automaticamente.

## Publicação autorizada — remoto GitHub

## Autorização vigente — Sprint 06, retomada 06.1

Em 03/11/2026 o usuário autorizou o Prompt 06.1, preservado em `docs/sprint06.1.original.md`: executar exclusivamente SM-026 a SM-028 de forma incremental, reaproveitando provas válidas. Checkpoint obrigatório em `docs/SPRINT_06_PROGRESS.md`; novas evidências em `docs/evidence/sprint06/`. Não refazer funcionalidades concluídas nem sobrescrever evidências históricas. Sem publicação de release/distribuição a terceiros ou V1.1 automática. Carga simulada não substitui Google real; baseline real continua humana. O usuário autorizou nesta retomada instalar Rust/C++/SDK oficiais e eventual UAC para recuperar o ambiente desta máquina.

### Registro histórico de publicação

Em 03/11/2026 o usuário autorizou expressamente criar e publicar todo o repositório em `https://github.com/kaykyDias04/sheets_monitor.git` (`origin`), incluindo `main` e todas as branches `codex/` por push normal, sem force-push. As menções anteriores a "sem remoto autorizado" são históricas. A publicação não altera o estado da Sprint 05 nem autoriza a Sprint 06. Aguardar Prompt 06.

## Transferência de contexto — fotografia verificada em 03/11/2026

Esta seção prevalece sobre estados históricos anteriores quando houver divergência. No checkout inspecionado, `HEAD` é `b2c68ce0791aa8acdcc860f433dd2ed6a61849da` (`docs(SM-022,SM-023,SM-024,SM-025): record full repository publication to GitHub`), branch local `main`; `origin/main` aponta para o mesmo commit. O remoto autorizado é `https://github.com/kaykyDias04/sheets_monitor.git`. A árvore Git estava limpa antes da tarefa de transferência. O histórico das branches `codex/` foi preservado. Não inferir CI hospedado aprovado apenas por existir o remoto/workflow.

Sprint 05 e SM-022–SM-025 estão concluídos tecnicamente; o próximo trabalho de produto só começa ao receber o Prompt 06. SM-026, SM-027 e SM-028 permanecem planejados. A pasta `docs/evidence/sprint05/` preserva testes e gates da sprint; esta tarefa registra novas verificações, sem sobrescrever aquela evidência histórica. O alerta WinRT de desenvolvimento foi confirmado pelo usuário; o primeiro teste negativo permanece registrado; o probe corrigido foi interrompido antes do relatório final. Não declarar coleta completa de callbacks/saída limpa para esse processo.

Restrições de continuidade: não iniciar Sprint 06 sem autorização; não aprovar ou alterar baselines reais, modificar arquivos Google, executar Apps Script, habilitar autostart, enviar novos toasts, alterar credenciais ou produzir/publicar instalador sem a autorização/escopo correspondente. Preservar os dados reais existentes e não exportá-los a logs/evidências. Migrations 0001–0005 são aditivas/históricas; nunca reescrever migration já aplicada. Manter Rust como dono de agendamento, fila, concorrência, política e integridade; React é cliente IPC e não implementa regras de domínio. Google permanece read-only; não exigir serviço hospedado.

Convenções: preservar IDs SM-001…SM-028 e subtarefas oficiais; mudanças de requisitos por decisão explícita/ADR; commits Conventional Commits ligados a SM, branches `codex/`; usar lockfile npm existente, sem migrar para pnpm implicitamente. Licença MIT cobre código próprio, dependências mantêm suas licenças. `archive/prototype/` é somente referência imutável. Antes de prosseguir, ler `docs/NEXT_STEPS.md`, `docs/ARCHITECTURE.md` (no checkout Windows, existente como `docs/architecture.md`), `docs/PROGRESS.md` (existente como `docs/progress.md`), `docs/sprint-05-report.md`, a especificação Sprint 06 assim que autorizada/fornecida, `docs/backlog.json`, `docs/risks.md`, `docs/adr/README.md` e `CONTRIBUTING.md`.

## Autorização vigente — Sprint 06

Em 03/11/2026, o usuário forneceu `docs/sprint06.original.md`, SHA-256 `34806E4FC5332E2BB89ADC54DF1A417E64E6397E1DC5CA994167F5D96F061EE0`. Executar exclusivamente SM-026–SM-028: auditoria, integração, segurança, carga5/15/30, correções, candidato1.0.0 e instalador local, documentação e homologação humana. Deepwork com lanes e gates. Não publicar release nem distribuir a terceiros.

Preservar migrações0001–0005, evidências históricas e alterações locais anteriores. Google somente leitura; baseline real humana; novos toasts/autostart dependem de autorização explícita. O usuário confirmou indisponibilidade de Windows separado e30 recursos reais com referências aprovadas: executar testes locais e registrar pendências. Não equiparar campanha sintética/relógio acelerado a Google real/24h. Instalação em outra pasta não isola cofre/perfil: não iniciar candidato instalado no perfil real; instalação/desinstalação limitada exige preflight sem colisões. Ao finalizar, relatório com critérios comprovados e pendentes e aguardar homologação final, sem iniciar expansão.

## Publicação técnica autorizada após a retomada 06.1

O usuário autorizou expressamente instalar o pacote e publicar a release no GitHub. Pré-release técnica v0.1.0-sprint06-preview publicada; instalação e startup verificados no limite registrado em docs/SPRINT_06_PROGRESS.md. Isso não aprova a Sprint 06, não autoriza baselines automáticas nem amplia o escopo para V1.1.


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.


### Atualização — audiência OAuth interna

O usuário informou que o console Google dispensa verificação porque o projeto está configurado com tipo de usuário Interno. Nesse cenário, não exigir aprovação pública como pré-condição: o OAuth limita-se aos membros da organização Google Cloud/Workspace do projeto e às políticas administrativas correspondentes. Fonte: https://support.google.com/cloud/answer/15549945#user-type-internal . A informação é relato do usuário; o nome/ID do projeto e o JSON Desktop ainda não foram fornecidos. O bloqueio atual é obter a configuração Desktop autorizada e comprovar login do candidato com conta interna elegível, além das homologações já pendentes. Não alterar a audiência para Externo nem ampliar escopos automaticamente. A futura release deve informar que o login é restrito à organização.

## Publicação autorizada após integração — pré-release1.0.0

O pedido atual autorizou integrar codex/sprint-06-homologation,recompilar com OAuth e publicar no GitHub. Após fornecer JSON Desktop do projeto interno autorizado,o usuário confirmou “Login e leitura funcionaram” no novo instalador. Publicada pré-release1.0.0,target b0ef2c6 com CI Windows completo aprovado,tag anotada e três assets com hashes conferidos. Evidências sanitizadas em docs/evidence/release-oauth. Não expor configuração privada,contas ou conteúdo real.

Esta publicação não classifica Sprint06 como APROVADA,não prova100% de funcionamento nem homologa carga Google real/24h/Windows limpo e demais critérios pendentes. Preservar esses limites,as evidências anteriores e o controle humano de baseline. Nenhuma expansão automática foi autorizada. O pacote mantém OAuth interno; verificação pública é dispensada nesse cenário,login restrito à organização. Novos toasts/autostart e mudanças reais continuam sujeitos às autorizações próprias.
