# Critérios de aceite e evidências

Somente evidência da aplicação ativa comprova a Sprint 01. Arquivos em `archive/prototype/`, estimativas do anexo e testes apenas previstos não representam conclusão.

## Sprint 01 — gate obrigatório

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
