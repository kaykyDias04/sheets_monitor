# Sprint 03 — encerramento técnico

SM-012 a SM-017 implementados e aprovados nos gates locais em 02/11/2026. Entrega: snapshots dirigidos, baselines imutáveis com aprovação humana, comparação estrutural e de conteúdo, fórmulas protegidas, análise AST Apps Script, classificação por aba/arquivo, eventos deduplicados e retenção SQLite. Interface técnica sob demanda, com isolamento de conta e geração de sessão. A migração 0003 preserva as anteriores e acrescenta sete tabelas, totalizando 22 tabelas de produto.

## Evidências verificáveis

| Verificação | Resultado | Registro |
|---|---|---|
| Rust | 146 unitários e 2 integrações aprovados; 1 teste de cofre ignorado na suíte padrão | [rust-test.log](rust-test.log) |
| React | 20 testes aprovados | [frontend-test.log](frontend-test.log) |
| Qualidade | Clippy com warnings proibidos, rustfmt, ESLint, Prettier e TypeScript aprovados | Logs rust-clippy, rust-format, frontend-lint, frontend-format e frontend-typecheck nesta pasta |
| Builds | React e executável MSVC com assets incorporados aprovados | [frontend-build.log](frontend-build.log), [rust-build.log](rust-build.log) |
| Cenários exigidos | 25 cenários mapeados aos testes determinísticos | [scenario-matrix.json](scenario-matrix.json) |
| Escala sintética | 30 arquivos × 1.000 registros; 30 diferenças; 3.458 ms; maior snapshot 489.737 bytes | [performance.log](performance.log) |
| Aplicativo nativo | Bridge, seis páginas, esquema 3 e diagnóstico encerrado | [native-qa.json](native-qa.json) |
| Acessibilidade | Axe sem violações na tela técnica vazia verificada | [native-technical-a11y.json](native-technical-a11y.json) |

O teste isolado do cofre Windows permanece comprovado pelo log da Sprint 02; seu código não foi alterado. Não foi contado como nova execução nesta sprint. Os logs Rust produzidos pelo Windows PowerShell usam UTF-16; os demais registros preservam sua codificação original. Falhas intermediárias foram preservadas e corrigidas antes dos resultados finais.

## Google real: cobertura e limites

O usuário aprovou uma baseline v1 restrita a **uma aba e duas células, sem regras de fórmulas nem scripts**. O build final recuperou a sessão, renovou o acesso, coletou esse escopo e confirmou Online; a repetição gerou zero eventos novos e preservou a baseline. Isso comprova o fluxo real desse escopo, sem inferir proteção de toda a planilha.

Uma captura adicional somente de leitura reuniu 800 células, 99 linhas observadas, duas regras críticas de fórmulas e um projeto associado por prova de vínculo, com 23 arquivos e 362 dependências. A análise AST foi parcial. Esse rascunho **não foi aprovado como baseline**, e não comprova comparação real das fórmulas protegidas. Contagens e resultados estão em [live-validation.json](live-validation.json); conteúdos e identificadores privados foram omitidos.

O ciclo destrutivo real de alterações, Offline e recuperação **permanece pendente**: foi verificado com fixtures, sem editar recursos Google. Nenhuma baseline real foi aprovada pelo agente. A concessão continua somente leitura. A conclusão dos IDs registra a implementação e o aceite técnico local, com essa pendência de homologação real explicitamente separada.

## Revisão e entrega

Gate 1 aprovado após corrigir cobertura parcial, referências de intervalos, journal de observação, validação de payloads, deduplicação e coerção de fórmulas. Gate 2 aprovado após corrigir corridas de sessão/prévia e criticidade das falhas opcionais. Gate 3 aprovado na primeira revisão final, sem achados materiais. As regressões fazem parte das suítes registradas. Não se declara auditoria externa.

Sem remoto Git autorizado: CI hospedado e publicação de PR não executados. O rascunho está em [pr-sprint03.md](../../pr-sprint03.md). Não foram implementados dashboard definitivo, scheduler, notificações, bandeja ou instalador. Próxima etapa: receber o Prompt 04 antes de iniciar SM-018 a SM-021.
