# Baseline — Sprint 06

Executado em 03/11/2026 na base b2c68ce, antes de mudanças de produto.

- lint, Prettier e typecheck: passaram (baseline-lint.log, baseline-format-check.log, baseline-typecheck.log).
- React paralelo:49/50, um timeout5s no teste de aprovação humana; falha preservada em baseline-test.log.
- React serial sem alterar testes, timeouts ou asserções:50/50 passaram em83.98s (baseline-test-serial.log). Limite observado de recursos/paralelismo do ambiente; não é prova de defeito do produto.
- Frontend build: passou (baseline-build.log).
- Rust:257 unitários,2 integrações passaram,1 teste de cofre ignorado por padrão. rustfmt e Clippy passaram (baseline-rust/).
- Nenhuma prova Google nova, toast, autostart ou aprovação de baseline real foi executada.

A execução serial será usada no gate final para limitar contenção da máquina. A falha paralela permanece visível; nenhum teste foi removido nem timeout ampliado.
