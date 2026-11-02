# Critérios de aceite e evidências

Somente evidência da aplicação ativa comprova a etapa correspondente. Arquivos em `archive/prototype/`, estimativas do anexo e testes apenas previstos não representam conclusão.

## Sprint 01 — gate histórico aprovado

| ID | Critério | Verificação mínima | Tarefas |
|---|---|---|---|
| AC-ENV | Windows 11 x64 apto a compilar Tauri | `scripts/doctor.ps1` com exit 0 e inventário | SM-001 |
| AC-REPO | Histórico preservado e governança consistente | Hashes do arquivo, links/IDs/templates, commit e status limpo | SM-002, SM-006 |
| AC-UI | Quatro telas e demonstração explícita | Testes de navegação/erros e inspeção da janela nativa | SM-003 |
| AC-NATIVE | Aplicativo abre no Windows 11 | Build MSVC e processo/janela com captura | SM-002, SM-004 |
| AC-DB | Onze entidades criadas e lidas | Testes Rust/integração, migrações idempotentes, reabertura e falha atômica | SM-005 |
| AC-PERSIST | Preferências persistem ao reiniciar | Salvar intervalo sintético, reabrir e conferir SQLite/UI | SM-005, SM-006 |
| AC-QUALITY | Lint, formato, tipos, testes e builds passam | Comandos encerrados com sucesso; clippy sem warnings | SM-006 |
| AC-SCOPE | Integrações futuras não apresentadas como prontas | Dois comandos foundation; módulos futuros são contratos sem rede | SM-004, SM-006 |
| AC-DATA | Sem credenciais ou dados reais versionados | Inspeção de padrões/diff sem imprimir segredos | SM-006 |

Comandos e resultados ficam em [progresso](progress.md), [desktop](desktop.md), [banco](database.md) e `docs/evidence/sprint01/`. Workflow configurado não equivale a CI remoto executado; isso exige um run verificável.

## Gates de produto para sprints posteriores

- Sprint 02: login autorizado, sessão no cofre e leituras reais Sheets/Drive/Apps Script. Fixtures cobrem falhas, sem substituir consentimento.
- Sprint 03: baseline aprovado/versionado, colunas/abas, fórmulas críticas, referências estáticas e quatro estados. Mudança permitida continua Online; cada diferença observada gera evento.
- Sprint 04: cadastro/edit/remove/pause, limite 30, seleção de abas, dashboard e histórico coletados.
- Sprint 05: polling 60–300s, até três avaliações, recuperação de rede/quota, notificações sem perda e operação minimizada/bandeja.
- Sprint 06: cenários integrados, carga de 30 arquivos com métricas, instalador/desinstalação Windows x64 e avisos de licenças/privacidade.

Cada gate futuro exige autorização e evidência própria. Aprovação de baseline, consentimento e homologação final pertencem ao usuário; não serão inferidos por tempo decorrido ou por sucesso de mock.

## Sprint 02 — gate aprovado

| Critério | Evidência real | Estado |
|---|---|---|
| SM-007 login e identidade | Conta conectada e UserInfo pelo backend de produção | Aprovado |
| SM-008 recuperação e renovação | Reinício com access token ausente e leituras seguintes com refresh protegido | Aprovado |
| SM-009 Sheets | Metadados/abas, valores não vazios, fórmula e valor calculado | Aprovado |
| SM-010 Drive | Metadados retornados com opt-in de leitura | Aprovado |
| SM-011 Apps Script | Metadados/fontes GS, HTML, JSON e hash válido | Aprovado |
| Qualidade local | Suítes React/Rust, Clippy, formato, tipos e builds atualizados | Aprovado |
| Privacidade | Evidências booleanas/contagens; sem identidade, IDs, tokens ou conteúdo | Aprovado no escopo verificado |

Relatório [Google real](evidence/sprint02/google-live.json) e [roteiro/resultados](evidence/sprint02/google-live.md). Capturas desconectadas anteriores são históricas; nenhum screenshot de dados reais foi feito. Revisão independente ficou indisponível, conforme [riscos](risks.md); não se declara auditoria externa ou CI hospedado.

Sem baseline aprovada, motor de integridade, scheduler ou notificações. Sprint 02 encerrada; aguardar Prompt 03.

## Sprint 03 — gate técnico aprovado, homologação real delimitada

| Critério | Evidência | Estado |
|---|---|---|
| SM-012 baseline versionado | Persistência, aprovação explícita, fingerprint e CAS; baseline humana real restrita a duas células | Aprovado no escopo verificado |
| SM-013 estrutura | Abas, colunas, cabeçalhos, intervalos nomeados e cobertura parcial em testes | Aprovado localmente |
| SM-014 fórmulas | Comparação exata/estrutural, movimento seguro, erros efetivos e ambiguidades | Aprovado com fixtures |
| SM-015 dependências | AST determinística, vínculos e incertezas; leitura real adicional com análise parcial | Aprovado localmente; limitações explícitas |
| SM-016 estados | Offline precede incerteza crítica; conectividade separada; Checking na interface | Aprovado com fixtures; Online real no escopo restrito |
| SM-017 eventos | Diferenças, observações parciais, deduplicação, recorrência, retenção e concorrência | Aprovado localmente; repetição real sem eventos novos |
| Qualidade | 146 unitários Rust, 2 integrações, 20 React, checks e builds | Aprovado |
| Mutações reais e recuperação | Não executadas; fixtures separadas | Pendente de recurso descartável e autorização |

A conclusão técnica não representa homologação destrutiva real nem aprovação de baseline pelo agente. Ver [relatório final](evidence/sprint03/final-report.md), [matriz](evidence/sprint03/scenario-matrix.json) e [leituras reais](evidence/sprint03/live-validation.json). Aguardar Prompt 04.

## Sprint 04 — gate técnico aprovado com cobertura real delimitada

| Critério | Resultado e limite | Evidência |
|---|---|---|
| SM-018 dashboard | Dados do backend, seis destinos e estados pendentes sem Online padrão | Testes React e native-qa.json |
| SM-019 descoberta/cadastro | Drive real paginado, cadastro por lista/URL e remoção/restauração local; limite30 transacional testado com fixtures | live-validation.json e testes Rust |
| SM-020 abas/regras | Seleção persistida, editor estruturado, proposta só rascunho, regressão31 abas; configuração ampliada atual ainda pendente | Testes188/41 e final-current-configuration.json |
| SM-021 atividades/diagnósticos | Filtros/paginação SQL e diagnóstico; consulta real vazia, histórico não vazio coberto por fixtures | scenario-matrix.json e live-management.log |
| SM-021A conta | Sessão recuperada, refresh real, troca de geração e sincronização avançada testadas | live-restart.log e testes React/Rust |
| SM-021B baselines | Aprovação explícita visual com fixtures; referência humana anterior preservada e reutilizada em leitura real | BaselineFlow/App tests e live-management.log |
| Windows 11/qualidade | 188 unitários Rust+2 integrações+42 React, builds/checks e seis rotas nativas aprovados | local-validation.json e gate3.md |
| Privacidade | Sem identificadores/conteúdo reais nos textos verificados; imagem pública redigida visualmente | private-identifier-scan.json e native-overview-redacted.png |

Aceite é técnico e não constitui homologação de toda a planilha real. A baseline humana cobre uma aba/duas células; a seleção atual de28 abas exige revisar regras e aprovar nova referência no aplicativo. Não houve nova aprovação real pelo agente, mutações destrutivas Google, CI hospedado, PR remoto ou publicação externa Google. Ver [relatório final](evidence/sprint04/final-report.md). Aguardar Prompt 05.
