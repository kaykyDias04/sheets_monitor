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

## Encerramento da Sprint 02 — Google real aprovado

Estado vigente: **Sprint 02 concluída**, SM-007 a SM-011 aprovados com recursos autorizados pelo usuário. Este encerramento substitui a pendência registrada acima; os registros anteriores permanecem históricos. O usuário conectou a conta e solicitou verificar as consultas efetuadas no aplicativo.

Comprovado no backend de produção: login/identidade, sessão protegida recuperada após reinício, access token inicialmente ausente da memória e renovado na primeira leitura sem novo consentimento. Sheets retornou metadados e 28 abas, valores não vazios e modo FORMULA. Uma consulta limitada a A1:H20 em 20 abas confirmou fórmula e valor calculado. Drive retornou metadados com opt-in; Apps Script retornou metadados e 23 arquivos GS/HTML/JSON, fontes e hash válido. Respostas e estado final conferem com a mesma conta, sem registrar a identidade.

Evidência sanitizada em docs/evidence/sprint02/google-live.json, google-live.md, google-live-read-results.log, google-formula-validation.log e google-final-build-results.log. Nenhum ID, email/nome, token, secret, célula, fórmula ou código foi gravado nesses arquivos. Não houve screenshot real, edição/execução Google, criação de projeto/faturamento nem revogação do consentimento.

Removido da interface o aviso fixo de homologação pendente e do DTO o campo de engenharia real_validation. Revalidação após a alteração: 13 testes React, 43 unitários Rust e 2 de integração aprovados; lint, Prettier, tipos, rustfmt, Clippy sem warnings e builds React/MSVC aprovados. O teste isolado do cofre de Windows continua comprovado pelo log anterior, com o mesmo código de cofre. Build atualizado testado novamente com sessão recuperada e leituras reais nas três APIs.

A instância diagnóstica foi fechada normalmente, após a revisão automática recusar o encerramento forçado. Porta 9227 confirmada fechada; app reaberto em modo normal com sessão preservada. A captura desconectada anterior continua histórica; hash do executável atualizado e fechamento do diagnóstico em google-authenticated-native.json.

Limites remanescentes: revisão independente indisponível, CI hospedado/PR remoto não executados por falta de remoto autorizado. Falhas de permissão/rede/revogação permanecem cobertas por testes controlados; a concessão real não foi revogada. Não há pendência humana para o aceite desta sprint. Rascunho PR atualizado em docs/pr-sprint02.md. **Encerrar e aguardar Prompt 03, sem implementar a Sprint 03.**

## Encerramento da Sprint 03 — aceite técnico local

SM-012 a SM-017 concluídos na implementação e nos gates locais. O [relatório final](evidence/sprint03/final-report.md) reúne resultados, limites e evidências. Este registro substitui o estado anterior de espera pelo Prompt 03; os registros das Sprints 01/02 permanecem históricos.

Implementados snapshots normalizados, aprovação humana de baselines versionados, diff de estrutura/conteúdo/fórmulas, AST Apps Script, classificação, eventos sem duplicação indevida, retenção e interface técnica. Migração 0003: sete tabelas novas, 22 de produto ao todo. Isolamento de conta/geração e CAS protegem aprovações e observações concorrentes.

Verificações: 146 testes unitários Rust e 2 integrações aprovados, 20 testes React aprovados, 25 cenários exigidos mapeados, lint/formato/tipos/Clippy e builds aprovados. Um teste de cofre permanece ignorado por padrão, com evidência explícita anterior e implementação inalterada. Escala sintética de 30 arquivos não equivale a carga Google real. Revisões dos três gates concluídas com correções e regressões incorporadas.

Leitura real no build final confirmou sessão recuperada, refresh e baseline v1 aprovada pelo usuário, restrita a uma aba e duas células, sem fórmulas protegidas/scripts. Online na repetição, zero eventos novos. Rascunho adicional de 800 células, duas regras de fórmulas e 23 arquivos de script foi capturado, mas não aprovado; AST parcial. Não declarar toda a planilha homologada.

Pendência: ciclo destrutivo Google real não executado, coberto por fixtures. Nenhum recurso Google foi alterado e nenhuma baseline real foi aprovada pelo agente. CI remoto e PR publicado indisponíveis sem remoto autorizado; rascunho em docs/pr-sprint03.md. Diagnóstico encerrado, janela normal reaberta. Encerrar e aguardar Prompt 04; não iniciar SM-018 a SM-021.

## Sprint 04 — execução autorizada em andamento

Anexo preservado em docs/sprint04.original.md, SHA256 4AE6D43AFBD53694CC1AEC38B485EEA584344BC5D6BC16950DA637C8E58DAB1F. Branch codex/sprint-04-interface-management. SM-018 a SM-021 e adições explícitas SM-021A/SM-021B; sem antecipar Sprint 05.

Fase 1 concluída no commit d6e9977: descoberta Drive, cadastro transacional por conta, migração 0004, histórico SQL e contratos operacionais. Gate 1 aprovado na segunda revisão após corrigir indicação da referência ativa entre escopos e referência histórica com configuração ainda ausente. Suíte completa: 180 unitários Rust e duas integrações aprovados, um teste de cofre ignorado por padrão; Clippy aprovado. Componentes iniciais: 32 testes React aprovados. Essas contagens são da fase 1, antes das adições seguintes.

Fase 2 em execução: rotas operacionais sem demonstração, editor visual, aprovação visual, histórico e conta/preferências. Sugestão de regras adicionada para as abas selecionadas com leitura de até 240 células de cabeçalho; cinco testes focados aprovados, sem salvar ou aprovar automaticamente. Testes de jornada e revisão Gate 2 em andamento. A homologação real e a conclusão desta sprint ainda não foram declaradas.
