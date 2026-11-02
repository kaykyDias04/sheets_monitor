# Contribuição e execução por etapa

O escopo do prompt autorizado limita o trabalho. A [especificação v1.0](docs/specification-v1.0.original.md) descreve o produto completo; isso não autoriza executar todas as sprints. Ao concluir uma etapa, registrar evidências, pendências e progresso e aguardar a próxima instrução. Consulte [AGENTS.md](AGENTS.md).

## Branches e commits

- Usar `codex/<etapa-ou-id>-<descricao>` em branches de trabalho.
- Preservar histórico; não fazer reset destrutivo, force-push ou apagar trabalho existente. Arquivar protótipos antes de substituí-los.
- Commits no formato `tipo(escopo): descrição [SM-XXX]`. Tipos: `feat`, `fix`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`.
- Exemplo: `test(database): verify migration rollback [SM-005]`.
- IDs SM-001 a SM-028 são os do documento original. Subtarefas usam `SM-005.01`; não renumerar para coincidir com prompts. Um prompt pode concluir apenas parte de uma tarefa.
- Revisão local é permitida sem remoto. Não publicar nem criar issues/PRs em conta ou repositório não autorizado.

## Evidências e qualidade

Na Sprint 01, executar lint, formatação, typecheck, testes Vitest e Rust, build frontend e compilação nativa MSVC. Comprovar janela Windows, navegação e SQLite persistente. Comandos e resultados ficam em `docs/evidence/sprint01/` e [progresso](docs/progress.md).

Não usar testes de mocks como prova de integração Google. Não transformar ausência de credenciais em sucesso fictício. Não alterar scripts, planilhas ou permissões reais. Autenticação, monitoramento real, bandeja, autostart e instalador pertencem a etapas posteriores.

## Dados e licenças

Nunca versionar tokens, segredos, arquivos OAuth, bancos reais ou relatórios com conteúdo privado. Fixtures devem ser sintéticas e identificadas. Os padrões de `.gitignore` ajudam, mas não substituem inspeção do diff.

O código próprio usa [MIT](LICENSE). Dependências conservam suas próprias licenças; revisar novos pacotes e manter os avisos exigidos antes da distribuição. Consulte [licenciamento](docs/licensing.md) e [segurança](SECURITY.md).

## GitHub

Não há remoto autorizado configurado. Para publicar posteriormente, o responsável pode instalar GitHub CLI e autenticar pelo navegador, sem colocar tokens em comandos ou arquivos:

```powershell
winget install --id GitHub.cli --exact --source winget
gh auth login --hostname github.com --git-protocol https --web
gh auth status
```

Depois de indicar o repositório correto e autorizar a publicação, configurar `origin` e enviar a branch. Nenhum remoto de outro projeto será reutilizado. Os templates locais já estão em `.github/`.
