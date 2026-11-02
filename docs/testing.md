# Verificação — Sprint 02

Executar no Windows 11 x64, na raiz do repositório:

```powershell
npm ci
npm run lint
npm run format:check
npm run typecheck
npm test
npm run build
.\scripts\check-rust.ps1 -Task format-check
.\scripts\check-rust.ps1 -Task test
.\scripts\check-rust.ps1 -Task clippy
.\scripts\check-rust.ps1 -Task build
node scripts/verify-repository.mjs
```

O helper Rust importa MSVC e escreve em `docs/evidence/sprint02/`; `-EvidenceDirectory` permite outro diretório. Não sobrescrever evidências da Sprint 01. Mocks e fixtures ficam em testes, sem comando IPC para habilitar dados falsos como resultado real.

A suíte Rust cobre PKCE/state/host/callback, consentimento negado, exchange/refresh com transporte simulado, invalidação e identidade, revogação explícita, segredo fora de DTO/Debug, parsers e respostas dos três serviços, hashes, erros/retries/timeouts/tamanho e persistência por conta. SQLite verifica upgrade da migração1 para2 e preservação das preferências; a migração1 permanece intacta. A suíte React verifica navegação da fundação, ações de conta, descarte de conta divergente, limpeza de segredo/resultados, erros fechados e confirmação de revogação.

Teste adicional do cofre Windows, com bytes sintéticos e namespace UUID exclusivo:

```powershell
. .\scripts\environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
cargo test --manifest-path src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc --features custom-protocol windows_vault_roundtrip -- --ignored
```

Esse teste exercita armazenamento real do Windows, substituição e limpeza apenas das entradas de teste. Não comprova tokens Google reais. O teste é ignorado na suíte padrão para evitar alterar cofre em execução não explícita.

Prova nativa deve abrir o executável com assets incorporados, confirmar bridge Tauri, migração2, cinco telas e estado desconectado sem fixtures Google. Capturas desconectadas podem ser versionadas. Instrumentação temporária de WebView deve escutar apenas loopback e ser encerrada ao terminar. Não registrar authorization URL, code, state, tokens, identidade ou conteúdo real em logs públicos.

Homologação externa separada: seguir [oauth-setup](oauth-setup.md), consentir no navegador, ler abas/valores/fórmulas de uma planilha fictícia autorizada, metadados Drive com opt-in, projeto Apps Script e fontes. Reiniciar e consultar novamente para renovar a sessão recuperada. Registrar só checks booleanos/categorias sanitizadas. Sem acesso autorizado, marcar pendente, nunca aprovado por mocks.

CI Windows usa os mesmos gates, mas existência do workflow não prova execução hospedada. As falhas iniciais ficam preservadas; resultados atuais e limitações constam em [progress](progress.md).

Executar o build standalone por último antes da prova nativa: `cargo test` com features padrão pode substituir o executável pelo modo development, que aponta para localhost:1420. O comando adicional de cofre acima usa custom-protocol para preservar esse modo. Não concluir sucesso standalone se o renderer depender do Vite.

A execução real desta sprint foi concluída e está em [google-live](evidence/sprint02/google-live.json), com o [relato sanitizado](evidence/sprint02/google-live.md). Os testes de recuperação e leituras foram repetidos no build atualizado, sem novo consentimento e sem expor dados privados.

## Sprint 03 — caminho de evidência

Baseline preservado:13 testes React e43 unitários Rust +2 de integração antes de implementar, em docs/evidence/sprint03/baseline*. Novas evidências não sobrescrevem as Sprints01/02. O helper Rust agora usa sprint03 por padrão.

A biblioteca sintética em src-tauri/src/integrity/fixtures.rs alimenta cenários determinísticos. Os testes do motor, parser, coletor, banco e runtime cobrem:

| Cenários obrigatórios | Evidência |
|---|---|
| Inserir/excluir linha e editar valor permitidos | engine_tests: keyed record/content e scale_tests |
| Reordenar/remover coluna, mudar cabeçalho, excluir/renomear aba | engine_tests: protected column/header/sheet |
| Apagar/substituir fórmula e erro efetivo | engine_tests: protected formula, REF/NULL_VALUE/LOADING |
| Deslocar fórmula por inserção/exclusão | engine_tests: unique key/target identity e casos ambíguos |
| Dependência estática válida/inválida/dinâmica | scripts::tests e engine_tests |
| Código/arquivos Apps Script alterados | engine_tests: script hash/file differences |
| Falha de autenticação/rede | auth tests e engine_tests: operational uncertainty |
| Persistência sem spam, recuperação e recorrência | integrity_store_tests: incident lifecycle/retenção |
| Baseline anterior e atualização indevida | integrity_store_tests e runtime_tests: confirmação, origem, CAS e imutabilidade |
| Múltiplas abas e Offline com inconclusões | engine_tests: aggregate/scope/precedence |
| Dados incompletos e offsets inválidos | collector::tests e google::grid::tests |
| Troca de sessão na prévia/aprovação | runtime_tests e IntegrityPage.test.tsx |
| Mudanças parciais A→B→B→A | integrity_store_tests: component journal/CAS |
| Escala30 arquivos×1000 registros | scale_tests com performance.log sanitizado |

Executar os checks de qualidade do início deste documento. Para registrar a medição sintética sem network:

```powershell
. .\scripts\environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
cargo test --manifest-path src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc thirty_workbooks_with_one_thousand_records -- --nocapture
```

A medição cobre algoritmos locais e tamanho serializado, não vazão de APIs para30 arquivos reais. Não impor prazo frágil de benchmark por máquina. Compilar custom-protocol por último antes da prova nativa.

A homologação real deve distinguir: leitura/captura autorizada, aprovação humana de baseline e mutações destrutivas em recurso descartável. Login da Sprint02 não autoriza aprovar referência. Sem recursos/autorização para mutações Google, executar esse ciclo com fixtures e marcar os passos reais como pendentes. Nunca ampliar escopos para escrita silenciosamente. Relatórios reais só podem conter indicadores/contagens, sem conta, IDs, valores, fórmulas ou fontes.

## Sprint 04 — verificação atual

Logs em docs/evidence/sprint04, sem sobrescrever sprints anteriores. Suíte final Rust188 unitários+2 integrações,1 cofre ignorado por padrão (prova explícita anterior inalterada); React42. Cenários requeridos24 mapeados em scenario-matrix.json. Rustfmt/Clippy/lint/tipos/Prettier e builds aprovados. Nova regressão verifica31 abas sem confundir limite30 arquivos; outra verifica abridor de URL canônica sem usar o abridor OAuth. Preview de script é testado antes da aprovação, sem reutilizar análise de projeto ausente.

A homologação real tem logs próprios de descoberta, gerenciamento, script, remoção e reinício; não é inferida por mocks. A referência real tem apenas duas células. Estado atual28 abas selecionadas ainda depende de novas regras/baseline. Novo fluxo de aprovação visual é automatizado somente com fixtures, sem aprovar produção. Histórico real vazio não substitui fixtures de eventos/falhas.

Relatórios nativos registram seis rotas com Axe sem violações no tema automático escuro, viewports medidos e captura com rótulos omitidos. Falhas intermediárias de expectativa, timeout de testes, contraste e bloqueio Windows do executável foram preservadas e corrigidas. Consulte native-tooling-notes.md para diferenças entre medição do renderer, ferramentas e funcionamento do aplicativo.
