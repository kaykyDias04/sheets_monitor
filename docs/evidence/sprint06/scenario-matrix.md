# SM-026 — matriz incremental

03/11/2026. Provas históricas preservadas; não confundir reutilização com nova execução.

| Cenário | Prova reaproveitada | Pendência desta homologação |
|---|---|---|
| OAuth, recuperação de cofre e refresh | `../sprint02/google-live.json`; implementação de cofre preservada | Confirmar sessão no aplicativo instalado nesta máquina, se disponível |
| Sheets, Drive e Apps Script somente leitura | `../sprint04/live-validation.json` e logs relacionados | Validar build instalado; nenhum recurso Google será modificado |
| Cadastro, seleção e baseline humana | `../sprint04/final-report.md` | Quatro cadastros anteriores pendentes; ampliar referência exige aprovação humana |
| Online/Offline/Verificando/Indeterminado e regras | `../sprint03/scenario-matrix.json`, `../sprint05/scenario-matrix.json`; `rust-test.log` nesta retomada | Regressão aprovada; Offline/mutações permanecem sintéticos |
| Estrutura, fórmulas e Apps Script | `../sprint03/final-report.md` | Não alegar mutações Google reais sem prova |
| SQLite, migrações e isolamento de conta | Duas integrações e testes de stores; `rust-test.log` desta retomada | Regressão aprovada; instalação ainda pendente |
| Outbox, deduplicação e recuperação | `../sprint05/scenario-matrix.json` | Regressão e identidade final instalada |
| Toast e execução oculta | `../sprint05/native-probe-validated.json`, `../sprint05/native-toast-winrt-observation.json` | Remetente instalado, clique físico e encerramento do probe de toast anterior não comprovados |
| Frontend operacional | `frontend-test.log`: 50 testes aprovados nesta retomada | Nenhuma alteração semântica no frontend |
| Carga local 30 arquivos | `load-test.log`, `load-validation.json`, `load-run.log`, `load-metrics.log` | Aprovada: 90 verificações, máximo três, 30 cotas/30 recuperações; relógio/transporte sintéticos |
| Carga Google de 30 planilhas | Nenhuma evidência recuperada | Recursos elegíveis/autorizados, baselines humanas e campanha real com métricas |
| Distribuição | `production-build.log`, `artifact-manifest.json`, `production-pe-headers.log` | Build técnico aprovado; instalação recusada na solicitação de execução, sem prova de persistência/desinstalação instalada |

Nenhum resultado de mock aprova uma linha de integração Google real. Não há duração mínima de soak especificada no Prompt 06.1 ou na tabela SM-027 recuperada; não inventar execução prolongada nem duração de aceite.
