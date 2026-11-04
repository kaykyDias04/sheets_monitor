# Requisitos canônicos — especificação v1.0

Fonte: [anexo original](specification-v1.0.original.md), preservado sem edição. SHA-256 `994A2FAA0E1D9C48450C5FE2B7B88AB9E237C17054EFE3A70E1EC291671F765E`. A interpretação anterior está em [requirements.prototype.md](requirements.prototype.md) e no arquivo do protótipo.

Sprints 01–05 encerradas tecnicamente, com limites reais documentados. A autorização vigente é o [Prompt 06](sprint06.original.md): SM-026–SM-028, homologação integrada, carga progressiva, segurança, candidato de produção e instalador local. A autorização não inclui publicação pública, release definitiva nem distribuição a terceiros.

Não atualizar baseline real automaticamente nem modificar Google. Agendador, notificações e bandeja são implementações históricas da Sprint 05; novos toasts e autostart continuam exigindo autorização explícita. Mocks/fixtures não comprovam APIs reais. Descoberta usa metadados autorizados e deve mostrar resultados incompletos; cadastro por URL permanece alternativo. O limite30 aplica-se aos cadastros presentes, incluindo pausados. O usuário informou que ainda não dispõe de ambiente Windows separado nem de 30 recursos com referências aprovadas; esses critérios não podem ser declarados comprovados pela campanha sintética.

## Produto V1 aprovado

| RF | Requisito | Backlog principal |
|---|---|---|
| RF-01 | OAuth 2.0 desktop com PKCE | SM-007, SM-008 |
| RF-02 | Cadastro por link | SM-019 |
| RF-03 | Editar, remover e pausar | SM-019 |
| RF-04 | Arquivo inteiro ou abas selecionadas | SM-020 |
| RF-05 | Baseline versionado com aprovação explícita | SM-012 |
| RF-06 | Colunas alteradas, excluídas e deslocadas | SM-013 |
| RF-07 | Alterações/erros em fórmulas protegidas | SM-014 |
| RF-08 | Apps Script autorizado e dependências estruturais estáticas | SM-011, SM-015 |
| RF-09 | Diferenças de linhas e valores | SM-017 |
| RF-10 | Notificar todas as diferenças observadas | SM-023 |
| RF-11 | Histórico com data, arquivo e diagnóstico | SM-021 |
| RF-12 | Dashboard de integridade | SM-018 |
| RF-13 | Monitoramento enquanto minimizado | SM-022, SM-024 |
| RF-14 | Recuperar status após correção | SM-016, SM-025 |

## Estados de integridade

- **Online:** avaliação concluída e integridade estrutural confirmada.
- **Offline:** falha confirmada em estrutura, fórmula protegida ou dependência crítica.
- **Verificando:** avaliação em execução.
- **Indeterminado:** não foi possível confirmar integridade por permissão, indisponibilidade ou falha técnica.

Alterar valor ou inserir/excluir linha comum gera evento e mantém Online quando permitido pelas regras. Coluna protegida reordenada/excluída, aba monitorada removida, fórmula crítica inválida ou dependência crítica quebrada gera Offline. Perda de conexão gera evento operacional e Indeterminado. Modificação de script depende do impacto; referência dinâmica não pode ser declarada válida automaticamente.

## Parâmetros e limites

| Parâmetro | Contrato |
|---|---|
| Sistema/distribuição | Windows 11 x64; Tauri 2; instalador `.exe` ou `.msi` em SM-028 |
| Capacidade | Até 30 arquivos cadastrados simultaneamente |
| Intervalo | Inicial 60 segundos; configurável de 60 a 300 |
| Paralelismo | Até 3 verificações |
| Histórico | Padrão de 90 dias |
| Baseline | Versionado; aprovação humana explícita |
| Notificações | Consolidação de repetidas sem perder eventos persistidos |
| Minimização | Sim, em etapa própria; autostart opcional |
| Alterações externas | Sem correção de planilhas e sem modificar/executar Apps Script |
| Infraestrutura | Local; sem servidor próprio e sem webapp hospedado |
| Sincronização entre PCs | Fora da V1 |

Informar quando tamanho/cotas tornam a frequência inviável. Não truncar respostas e declarar sucesso. Polling registra somente diferenças efetivamente observadas, sem garantia de capturar toda edição intermediária.

## Banco previsto

`accounts`, `spreadsheets`, `monitored_sheets`, `baselines`, `integrity_rules`, `script_projects`, `script_dependencies`, `monitoring_checks`, `change_events`, `notifications` e `app_settings`. Migrações devem comprovar criação e leitura; o ledger interno não substitui nenhuma entidade.

## Apps Script a partir da V1

Ler somente projetos autorizados por ID, comparar fontes com referência e reconhecer referências estáticas `getSheetByName()`, `getRange()` e índices de colunas. Verificar a estrutura referenciada e sinalizar dependências dinâmicas não verificáveis. Indicadores genéricos de texto do protótipo não satisfazem, sozinhos, esse requisito. Nenhum código será executado ou alterado pelo monitor.

Critérios em [acceptance.md](acceptance.md), tarefas oficiais em [backlog.md](backlog.md), decisões em [adr/README.md](adr/README.md) e evidência real em [progress.md](progress.md).
