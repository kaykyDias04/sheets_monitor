# Progresso por etapa

## Prompt 00 — contrato

Concluido: contrato registrado em `AGENTS.md`. Implementacao anterior interrompida e arquivos preservados. O contrato revoga o avanco automatico entre etapas.

## Prompt 01 — ambiente Windows

Concluido. Escopo: inventario, dependencias de desenvolvimento, PATH, bootstrap e doctor. Nenhuma nova funcionalidade do produto foi implementada nesta etapa.

Ja preparado: scripts idempotentes `scripts/bootstrap.ps1`, `scripts/doctor.ps1` e helpers; inventario automatico `docs/environment.md`; politica/fontes `docs/environment-policy.md`; evidencias em `docs/evidence/environment/`.

O diagnostico inicial retornou 1 por ausencia de C++/SDK e componentes Rust. pnpm instalado no perfil do usuario; rustfmt/clippy instalados; PATH Rust corrigido sem duplicacao. Instalador C++ executado e retomado com consentimento UAC. As tentativas reportaram cancelamento nos logs, mas o estado efetivo final foi instalado/completo e sem reinicio pendente; o aceite foi estabelecido pelas provas de compilacao e execucao, nao por presumir sucesso do instalador. Nenhum comando de desinstalacao foi executado.

Aceite: `scripts/doctor.ps1` retornou **0**, com 16 verificacoes positivas. Compilacao/execucao de C++ com headers e bibliotecas Windows, recurso `.res`, Rust MSVC e Tauri/Wry passaram. Versoes e caminhos estao em `docs/environment.md`; Tauri da probe 2.12.5 e CLI 2.12.4. Bootstrap completo com doctor passou em Windows PowerShell 5.1; doctor direto passou em PowerShell 7. O ajuste de PATH e a repeticao do bootstrap preservaram as instalacoes e o PATH ja configurados.

Correcoes verificadas: aspas na chamada VsDevCmd; manifesto Common Controls 6 na probe Tauri (a primeira versao compilava, mas falhava no loader). Falhas iniciais e evidencias finais foram preservadas em `docs/evidence/environment/`. Analise de sintaxe PS 5.1/7 e guardas de codigo de saida/hash tambem passaram.

Pendencias desta etapa: nenhuma. Build funcional/instalador do produto, Google/OAuth e baselines nao foram validados e dependem dos prompts correspondentes.

Proxima etapa somente apos concluir este aceite e receber o **Prompt 02**. Nenhum avanco automatico.
