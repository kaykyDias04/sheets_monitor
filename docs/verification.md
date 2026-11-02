> DOCUMENTO HISTÓRICO DO PROTÓTIPO. Este texto descreve código arquivado, não a fundação ativa nem uma integração real homologada. Consulte [requisitos atuais](requirements.md), [arquitetura](architecture.md) e [progresso](progress.md). A versão original permanece em `archive/prototype/` e no commit `ffcaa49`.

# Plano de verificação

Responsável pela reconciliação: agente principal. Implementadores verificam seus módulos e um revisor independente examina cada fase.

| Afirmação | Evidência mínima | Limite |
|---|---|---|
| Até 30 monitores, baseline aprovada e diffs sem supressão | testes de domínio e SQLite em memória/arquivo | não prova cobertura de campos ausentes na API |
| Histórico/outbox atômicos, falha preserva estado | testes de transação e reabertura | entrega Windows depende do SO |
| OAuth desktop e Google somente leitura | testes PKCE/callback/transporte, revisão de endpoints | consentimento real não exercido sem credenciais |
| Scripts V1 | fixtures fonte/manifest e achados | análise heurística, sem execução |
| Interface utilizável | testes React, typecheck e build | QA nativo depende do toolchain e WebView |
| Desktop compila | cargo test/check e build Tauri com logs | não alegar execução se bloqueado |

Não expor tokens, conteúdo de respostas reais, IDs privados ou credenciais nos logs de evidência. Fixtures usam dados sintéticos. Testes são repetidos após mudanças relevantes ao comportamento coberto.
