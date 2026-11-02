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

## Sprint 01 do anexo v1.0 — autorização substitutiva

Durante o Prompt 02, o usuário forneceu a especificação fechada e escolheu explicitamente substituir a etapa atual pela Sprint 01 do anexo. O registro anterior de ambiente permanece histórico; a autorização vigente inclui fundação executável e termina antes da Sprint 02.

Estado: **Sprint 01 concluída tecnicamente**. Protótipo preservado em `ffcaa49` e em `archive/prototype/` (47 hashes conferidos). IDs oficiais SM-001 a SM-028 mantidos, com 33 subtarefas e dependências sem ciclos. Licença própria MIT e templates configurados; nenhum remoto foi criado ou publicado.

Gates já executados:

- SQLite MSVC: 11 unitários e 2 de integração, aprovados, incluindo onze entidades, migrações, rollback, capacidade e reabertura.
- Rust desktop: 12 unitários e 2 de integração, aprovados; rustfmt e clippy sem warnings aprovados.
- Frontend: instalação limpa, format:check, typecheck, ESLint, oito testes Vitest e build aprovados; audit sem vulnerabilidades reportadas nesta resolução.
- Repositório: especificação original, backlog, hashes do arquivo, links, templates, licença e padrões conhecidos de credenciais verificados. Esse scan não equivale a uma auditoria completa de segredos.

Gate final nativo: executável standalone MSVC x64 com assets incorporados, janela Windows e bridge Tauri confirmados; quatro telas navegadas, preferência60 ->120 persistida após reabertura e restaurada60; onze tabelas, quatro fixtures e seed sem duplicação. Axe4.12.1 retornou zero violações/itens incompletos nas quatro telas após corrigir contraste. A sessão diagnóstica foi encerrada e a aplicação reaberta normalmente, sem Vite ou porta de debug. Relatórios e limites em `docs/evidence/sprint01/native-qa.json` e `native-qa.md`. O workflow Windows foi configurado; CI GitHub não foi executado por não haver remoto autorizado.

## Encerramento da Sprint 01

SM-001 a SM-006 concluídos com evidências vinculadas no backlog. SM-007 a SM-028 permanecem planejados. A revisão independente reconciliou esquema/DTO/documentação; achados de FK, foco e contraste foram corrigidos e verificados. O helper de captura Windows retornou conteúdo fora do alvo e foi descartado: as imagens versionadas são do renderer WebView2 da instância nativa, sem a moldura do Windows. Não são screenshots da prévia Vite.

Preservação: histórico de ambiente `46ae4fd`, snapshot original `ffcaa49`, fundação `a0f2a5d` e commits corretivos/finais consultáveis com `git log`. Fontes arquivadas conservam os47 hashes originais. Artefatos gerados (`target/`, `dist/`, schemas Tauri) ficam fora do versionamento; logs e relatórios sintéticos permanecem em docs.

Pendências desta entrega: nenhuma técnica de fundação. Limites externos: integração Google, notificações, bandeja, autostart, carga30, instalador e homologação final do produto não executados. Inventário de licenças registra lacunas de dependências não presentes no cache; avisos de distribuição pertencem a SM-028. Nenhum segredo, recurso Google, remoto GitHub ou CI hospedado foi criado.

**Próximo passo: aguardar nova instrução para a Sprint 02 do anexo (SM-007 a SM-011). Não avançar automaticamente.**

## Sprint 02 — implementação local e checkpoint Google

Estado atual: **implementação e gates locais aprovados; homologação real pendente**. O usuário confirmou que configurará o cliente Desktop e autorizará testes. Ainda não existe evidência de login/refresh Google real nem de recursos autorizados das três APIs. SM-007 a SM-011 permanecem bloqueados no aceite principal; subtarefas locais concluídas e evidências separadas.

Implementado: OAuth Desktop com PKCE S256, callback IPv4 loopback em porta dinâmica, state/parâmetros/host/prazo validados, navegador externo, UserInfo, sessão no cofre Windows, access token somente em memória, refresh limitado, logout e revogação explícita. Concessões efetivas e geração protegem contra escopos ausentes, troca de conta e respostas tardias. Nove comandos nativos ao todo, com rede/cofre fora da thread UI.

Leitores tipados: metadados/abas Sheets, batchGet de intervalos calculados e FORMULA, metadados Drive opt-in e Apps Script projects.get/getContent com arquivos SERVER_JS/HTML/JSON e hash local. Sem executar ou editar scripts. Associação planilha/projeto exige escolha explícita e consulta prévia da mesma conta. Drive usa drive.metadata.readonly, sem permissão de escrita; nova autorização completa substitui incremental incompatível com apps instalados, conforme ADR0007.

Persistência: migração0002 preserva0001 e adiciona quatro tabelas de configuração/recursos/consultas/associações, mais metadados de conta. Não há tokens/secrets, células, fórmulas ou fontes no SQLite. Consultas manuais não criam monitores, eventos nem baseline. UI mostra loading/erros/conta/escopos/resultados independentes; limpa segredo do campo e descarta resultados de outra sessão. As telas da fundação continuam demonstrativas.

Evidências executadas em docs/evidence/sprint02:

- React:13 testes aprovados; ESLint, Prettier, TypeScript e build Vite aprovados.
- Rust MSVC:43 unitários e2 de integração aprovados;1 teste de cofre ignorado na suíte padrão e executado explicitamente com sucesso. Rustfmt, Clippy sem warnings e build standalone aprovados.
- Testes de rede simulada: parsers/formatos, PKCE/callback/consentimento, refresh/invalid_grant/identidade, logout concorrente,401 limitado,400/403/404/429/5xx, timeout/rede, respostas grandes/redirects, hashes e segregação de conta. Não são prova de Google real.
- Windows Credential Manager real: roundtrip com chunks sintéticos, substituição e limpeza do namespace exclusivo aprovados.
- Nativo: renderer tauri.localhost, bridge, cinco telas, migração2 com quatro fixtures e intervalo60 preservados, Client ID inválido rejeitado sem persistir. Axe4.12.1 na tela Google com0 violações/incompletos. Porta de diagnóstico encerrada e app reaberto normalmente.
- Governança:28 IDs oficiais,38 subtarefas, hashes dos dois anexos e47 arquivos do protótipo preservados, links/templates/escopo/Unicode/padrões conhecidos de credenciais verificados. npm audit de produção reportou0 vulnerabilidades; não cobre toda a supply chain.

Falhas iniciais preservadas: expectativas de versão/título da Sprint01, lint do hook e Clippy corrigidos; probe nativa development após cargo test foi corrigida com build custom-protocol por último. Nenhuma falha foi apresentada como sucesso. Revisão local pelo principal está em local-review.md; indisponibilidade das lanes impede afirmar revisão independente. Histórico/evidências da Sprint01 preservados.

Próximo passo: usuário configurar o cliente no app, consentir no navegador e consultar planilha/script de teste; depois comprovar reinício/refresh e registrar resultados sanitizados. Seguir docs/oauth-setup.md e docs/testing.md. Nenhum projeto/API/faturamento foi alterado pelo agente. Sem remoto autorizado, PR preparado apenas em docs/pr-sprint02.md e trabalho em branch codex/sprint-02-google-integration. Não iniciar Sprint03; Prompt03 depende de instrução posterior e não substitui a pendência de homologação atual.
