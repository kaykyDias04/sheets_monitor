# Progresso por etapa

## Sprint 06 — retomada incremental autorizada em 03/11/2026

Estado **PENDENTE DE HOMOLOGAÇÃO**. Prompt 06.1 preservado em `sprint06.1.original.md`; base `b2c68ce`, branch `codex/sprint-06-homologation`. Não foi encontrado trabalho anterior da Sprint 06 neste checkout. Evidências históricas permanecem preservadas. Checkpoint corrente e próxima ação: [SPRINT_06_PROGRESS.md](SPRINT_06_PROGRESS.md).

Frontend: 50 testes, tipos, lint, formatação normalizada e build produção aprovados. Rust: 258 unitários, duas integrações, um cofre ignorado por padrão, Clippy/rustfmt aprovados. Ambiente restaurado com autorização humana. Carga sintética de 30 arquivos aprovada; instalador técnico x64 gerado com hashes, sem configuração OAuth. Instalação recusada na solicitação de execução e não realizada. Carga Google real/validação instalada permanecem pendentes. [Relatório](sprint-06-report.md). Não publicar release nem iniciar V1.1.

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

## Sprint 04 — encerramento técnico aprovado

Estado vigente: SM-018 a SM-021 e adições SM-021A/SM-021B concluídos tecnicamente. O [relatório final](evidence/sprint04/final-report.md) substitui o estado de execução registrado acima e detalha mudanças, comandos, migração e limites. Gate 1 aprovado na segunda revisão, Gate 2 na terceira e Gate 3 na primeira, sem achados materiais pendentes.

Verificações finais: 188 unitários Rust, duas integrações e 42 React aprovados; Clippy, formato, lint, tipos e builds aprovados. Execução nativa com seis rotas, schema 4/24 tabelas e Axe sem violações/incompletos no tema automático escuro. Viewports780/1280 conferidos sem overflow horizontal da página. Captura pública teve rótulos e inicial da conta omitidos. Diagnóstico encerrado, porta 9234 fechada, fechamento normal sem diagnóstico comprovado e janela normal reaberta.

Google real: duas páginas Drive, cadastro pela lista e por URL, metadados de28 abas, proposta sem salvamento, referência humana de duas células verificada Online duas vezes sem eventos novos na repetição. AppsScript retornou23 arquivos com parentId e associação. Pausa/retomada, remoção/restauração local, consulta de histórico vazio, comparação e reinício foram comprovados. Intervalo de teste persistiu e foi restaurado para 60.

Após as verificações, a seleção atual passou para28 abas com escopo de proteção ainda de uma aba. O status correto é Configuração pendente; essa escolha foi preservada. Nenhuma referência nova foi aprovada pelo agente e nenhum arquivo Google foi alterado. Nova aprovação visual real e mutações destrutivas continuam explicitamente fora da homologação executada; fluxo visual e cenários de falhas estão cobertos por fixtures. A publicação externa Google depende do projeto/provisionamento/verificação pertinentes.

Uma instância diagnóstica sem janela precisou ser encerrada após tentativas normais para liberar o executável; somente o PID/caminho de teste foi atingido. A falha de build está preservada e o rebuild final passou. Detalhes em native-tooling-notes.md. A instância diagnóstica final e o probe sem diagnóstico fecharam normalmente. Evidências das sprints anteriores permanecem intactas.

Commits das fases d6e9977/f84efae; fechamento em commit final na mesma branch codex/sprint-04-interface-management. Sem remoto autorizado: somente rascunho PR local. Encerrar e aguardar Prompt 05; não executar Sprint 05.

A preferência padrão de histórico agora alimenta novas regras e a proposta quando não existe escopo salvo. Escopos e rascunhos existentes mantêm seus prazos. Nova regressão e prova nativa confirmaram365 dias em um novo rascunho, restauração do padrão 90 e ausência de mudanças nas regras/seleções salvas. Totais finais188 unitários Rust+2 integrações+42 React; artefato atualizado em native-qa.json. Após o ajuste, a instância diagnóstica fechou normalmente, porta 9234 foi confirmada fechada e a janela normal foi reaberta. O estado mais recente observado contém 4 cadastros pendentes, com alterações externas aos testes preservadas.
## Sprint 05 — execução autorizada em andamento

Autorização: `docs/sprint05.original.md`, SHA256 `4681BC47821FA65E35DE208207C22F7B6A61DCD78095E42173C3D838EA34C08F`. Branch `codex/sprint-05-continuous-monitoring`, base Sprint 04 `4041b71`. Implementar somente SM-022 a SM-025. Sem instalador, campanha final de 30 planilhas, escrita Google ou aprovação automática de baseline.

Baseline anterior reproduzido antes das mudanças: 188 unitários Rust e duas integrações, 42 testes React. Gate 0 aceito com contratos explícitos. A fase de persistência passou 71 testes focados; a interface passou 45 testes após os primeiros controles. Gate 1 encontrou correções necessárias no embargo de Retry-After, na guarda de sessão ao concluir falhas e na recuperação de falha de criação de worker. Implementação e verificação continuam; nenhum aceite nativo ou conclusão da Sprint 05 é inferido deste registro. Evidências novas ficam em `docs/evidence/sprint05/`.

## Sprint 05 — encerramento técnico

SM-022 a SM-025 concluídos tecnicamente. O [relatório](sprint-05-report.md) substitui o estado de execução acima. Fases iniciais em `dac71dc` e `23e781e`; correção Windows e fechamento em commits seguintes da mesma branch `codex/sprint-05-continuous-monitoring`.

Suíte final: 257 unitários Rust, duas integrações e 50 React aprovados; um teste de cofre ignorado por padrão e evidência histórica preservada. Lint, formato, tipos, Clippy e build Tauri MSVC com assets incorporados aprovados. O probe nativo comprovou verificações ocultas, intervalo 60/300s, pausa/retomada e saída limitada com recursos sintéticos. O aplicativo real comprovou início/pausa, ocultação, instância única/restauração e saída voluntária.

A primeira tentativa de alerta não foi exibida; após a correção WinRT, o usuário confirmou a exibição do segundo alerta autorizado. Preservadas as duas observações e a interrupção do probe corrigido antes do JSON/callbacks finais; sem alegação de coleta completa ou saída limpa nessa execução específica. Fonte, mock, processo Windows e confirmação humana permanecem separados.

Quatro cadastros reais pendentes e zero proteções elegíveis foram preservados durante a inspeção. Nenhuma nova baseline real, escrita Google, ativação autostart, instalador ou campanha final de 30 arquivos foi realizada pelo agente. Identidade instalada e cliques físicos permanecem fora da homologação executada. Sem remoto autorizado: PR apenas em rascunho local. Próxima etapa: aguardar Prompt06, sem executar a Sprint06.


## Publicação no GitHub autorizada

Em 03/11/2026 o usuário autorizou criar e publicar o repositório `https://github.com/kaykyDias04/sheets_monitor.git`, enviando `main` e todas as branches `codex/` por push normal, sem force-push. Antes do envio: verificador do repositório, typecheck, lint e Vitest reexecutados; evidências históricas da Sprint 05 não foram regravadas. Estado das sprints inalterado; aguardar Prompt 06.

## Fotografia atual para transferência de conversa — 03/11/2026

Este registro prevalece sobre os estados e limitações históricos acima quando eles divergirem. checkout inspecionado: branch `main`, `HEAD` e `origin/main` em `b2c68ce0791aa8acdcc860f433dd2ed6a61849da`; remoto `origin` aponta a `https://github.com/kaykyDias04/sheets_monitor.git`. `git status --short --branch` estava limpo antes desta tarefa. As branches `codex/` permanecem preservadas/publicadas. Não foi inferido resultado de CI hospedado.

As Sprints 01–05 e SM-001–SM-025 estão registradas como concluídas conforme seus escopos e evidências (incluindo as adições SM-021A/SM-021B). A Sprint 05 está tecnicamente concluída; os detalhes, limites e erros já corrigidos estão em [relatório da Sprint 05](sprint-05-report.md) e [gate final](evidence/sprint05/gate3.md). SM-026, SM-027 e SM-028 continuam planejados; a próxima etapa permitida é aguardar e executar apenas o Prompt 06, depois de recebido. Não implementar nada da Sprint 06 por inferência.

Esta tarefa de continuidade executou novas verificações; resultados e logs estão em [`docs/evidence/context-transfer/`](evidence/context-transfer/README.md), distintos das evidências finais históricas da Sprint 05. Passaram lint, Prettier check, typecheck, build Vite, 50 testes React, 257 testes unitários Rust, duas integrações Rust, rustfmt check, Clippy, build Tauri MSVC e verificador do repositório. Um teste Rust de cofre continuou `ignored` como previsto. A primeira execução do verificador coincidiu com a criação do documento NEXT_STEPS e observou um link ainda não presente; a repetição após estabilizar os documentos passou. O verificador escreve por padrão no JSON histórico da Sprint 05; o original foi restaurado após a execução, e o log da repetição foi mantido em `context-transfer/`.

Para iniciar uma nova conversa, leia primeiro [instruções](../AGENTS.md), [próximos passos](NEXT_STEPS.md), [arquitetura](architecture.md), este arquivo, [relatório Sprint 05](sprint-05-report.md), [backlog](backlog.json), [riscos](risks.md), [decisões](adr/README.md) e [contribuição](../CONTRIBUTING.md). Os nomes pedidos `docs/PROGRESS.md` e `docs/ARCHITECTURE.md` resolvem para estes arquivos rastreados como `docs/progress.md` e `docs/architecture.md`; evite entradas Git que diferem apenas por capitalização.

## Sprint 06 — execução autorizada

Prompt recebido em03/11/2026 e preservado em `sprint06.original.md`. Branch `codex/sprint-06-homologation`, base `b2c68ce`. Alterações documentais da transferência anterior foram preservadas, sem atribuí-las à sprint. Auditoria inicial/gate1 e baseline em `evidence/sprint06/`; commit `2664ed9`.

Baseline atual:257 unitários Rust e2 integrações passaram,1cofre ignored; fmt/Clippy/lint/Prettier/typecheck/build frontend passaram. React paralelo49/50 com um timeout5s; repetição sem alterar testes/timeouts, um worker:50/50 em83.98s. Falha e sucesso preservados em logs separados. Campanha integrada e empacotamento em andamento; não inferir conclusão.

Usuário confirmou ausência de Windows separado e30 recursos reais aprovados. Executar campanha sintética local, mantendo carga Google/24h/execução limpa e OAuth do candidato como pendências até prova. Nenhuma baseline real pelo agente, escrita Google, toast ou autostart nesta sprint. Não publicar/distribuir a terceiros. O próximo checkpoint é homologação humana da V1.0, sem Sprint07 automática.

## Atualização autorizada — instalação e pré-release

O usuário autorizou expressamente instalar o pacote e publicar a release no GitHub nesta conversa. Essa autorização substitui a recusa de instalação anterior e a restrição anterior de publicação para este pacote técnico.

Instalado por usuário em `%LOCALAPPDATA%\Programs\Sheets Monitor`. Processo permaneceu ativo após abertura com Node/Rust/MSVC fora do PATH; avisos de terceiros, biblioteca padrão Rust e manual presentes. A comparação integral do payload confirmou diferença exclusiva do marcador Tauri `NSS` (NSIS) versus `UNK` (executável solto); hashes e resultado em `docs/evidence/sprint06/installed-payload-validation.json` e `install-authorized-result.json`. A comparação inicial de hashes falhou por esse marcador; não houve alteração de código ou rebuild.

Pré-release publicada: https://github.com/kaykyDias04/sheets_monitor/releases/tag/v0.1.0-sprint06-preview . Instalador e SHA256SUMS.txt enviados, com digests retornados pelo GitHub conferidos. Registro em `docs/evidence/sprint06/github-release.json`. Tag aponta para `f19f1ad`, fonte do pacote. Não é aprovação da V1.0: OAuth ausente, carga Google real, persistência após reinício, desinstalação e homologação Windows completa continuam pendentes. Nenhuma baseline aprovada, autostart ativado ou arquivo Google alterado.

Próxima ação: disponibilizar a configuração OAuth da aplicação por caminho local seguro e concluir a homologação pendente. Preservar o pacote instalado; não reinstalar nem reconstruir sem necessidade.


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.

Compilação atual concluída: candidato NSIS Windows x64 1.0.0, sem cliente OAuth incorporado e sem assinatura. Artefato/hashes e testes em [relatório da integração](evidence/release-integration/report.md). A geração do instalador não conclui homologação nem autoriza afirmar OAuth verificado. Publicação funcional continua pendente da configuração/projeto Google e das provas reais. Não houve instalação/abertura deste novo candidato no perfil real.


### Atualização — audiência OAuth interna

O usuário informou que o console Google dispensa verificação porque o projeto está configurado com tipo de usuário Interno. Nesse cenário, não exigir aprovação pública como pré-condição: o OAuth limita-se aos membros da organização Google Cloud/Workspace do projeto e às políticas administrativas correspondentes. Fonte: https://support.google.com/cloud/answer/15549945#user-type-internal . A informação é relato do usuário; o nome/ID do projeto e o JSON Desktop ainda não foram fornecidos. O bloqueio atual é obter a configuração Desktop autorizada e comprovar login do candidato com conta interna elegível, além das homologações já pendentes. Não alterar a audiência para Externo nem ampliar escopos automaticamente. A futura release deve informar que o login é restrito à organização.

## Candidato recompilado com OAuth interno — 03/11/2026

O usuário identificou o projeto autorizado e disponibilizou seu JSON Desktop em Downloads. Tipo Desktop e correspondência do projeto conferidos localmente, sem registrar conteúdo/credenciais. Recompilado o código de main b0ef2c6 com configuração OAuth no ambiente Process, sem alterar escopos, perfil ou cofre. Presença da configuração no binário confirmada por comparação em memória. Isso substitui a ausência de provisão nos candidatos anteriores; não substitui login real.

Instalador1.0.0,manifesto e hashes enviados ao rascunho GitHub; arquivo local e digest remoto conferidos. Relatório em [candidato OAuth](evidence/release-oauth/report.md). Audiência interna limita login à organização; dispensa verificação pública Google conforme informação do usuário. CI da fonte e homologação humana do novo pacote continuam condições separadas. O agente não abriu o candidato no perfil real nem declarou100% de funcionamento. Publicação definitiva aguarda os critérios de aceite e a validação humana solicitada. Nenhuma tag definitiva publicada.

CI da fonte b0ef2c6 concluído com sucesso, incluindo todos os passos Windows. Evidência sanitizada em `docs/evidence/release-oauth/ci-source.json`. Novos commits de evidência não representam nova execução nem CI exato em seus próprios heads. Rascunho1.0.0 com OAuth interno permanece aguardando homologação humana; nenhum release definitivo/tag publicado.

## Publicação da pré-release1.0.0

Usuário confirmou “Login e leitura funcionaram” no novo instalador. Evidência humana específica registrada em `docs/evidence/release-oauth/human-confirmation.json`; não equivale à homologação completa. Publicada a pré-release autorizada [v1.0.0](https://github.com/kaykyDias04/sheets_monitor/releases/tag/v1.0.0), com OAuth interno,NSIS,checksums e manifesto. Tag anotada aponta para b0ef2c6,fonte com CI Windows completo aprovado. Digests dos três assets conferidos na API GitHub.

Sprint06 permanece pendente de homologação integral,sem afirmação stable/100%. Carga Google real/24h,Windows limpo e demais critérios reais documentados continuam pendentes; não iniciar expansão automaticamente. O download público do binário não amplia a audiência do OAuth,restrita à organização. Relatório de publicação e limpeza em `docs/evidence/release-oauth/`.

## Melhorias de monitoramento — 13/11/2026

Pedido autorizado: exportação filtrada, autoria conservadora com auditoria opcional, intervalo até 600s, erros e cotas contextualizados, sincronização por conta Google e atividades restritas às regras. O trabalho anterior da branch `codex/monitor-improvements` foi preservado.

A sincronização usa commits imutáveis na pasta privada do aplicativo no Drive, backup local e decisão explícita de conflitos vinculada ao estado exibido. Migração aditiva 0007 preserva a versão de origem das referências e permite ordinais locais distintos sem reescrever UUIDs/histórico. A restauração descarta observações anteriores e não fabrica consultas ou permissões Google.

Validação local: Vitest completo com 103 testes aprovados; Rust MSVC com 298 unitários e duas integrações aprovados, mais um teste de cofre ignorado. Após a correção do DTO, passaram dez testes React focais e uma regressão Rust adicional. Typecheck, ESLint, Prettier, Rustfmt, Clippy, verificador do repositório, build frontend e build Windows `custom-protocol` aprovados. O executável foi recompilado após o build final do cliente para incorporar os assets atuais. Logs locais em `%TEMP%/sheets-monitor-validation-current`.

A primeira validação encontrou uma contagem legada fixa de comandos IPC no verificador e uma inicialização de fixture rejeitada pelo Clippy. A asserção de contagem incidental foi removida, sem substituí-la por outra contagem fixa; a fixture passou a inicializar os campos diretamente. Os dois checks passaram após essas correções. A revisão independente encontrou perda da categoria visual ao classificar mensagens traduzidas; o DTO agora preserva `diagnostic_code`, comprovado nas regressões Rust e React.

A prévia web real abriu Configurações e Atividades sem exceções JavaScript observadas. Um cenário temporário com os componentes reais e dados sintéticos comprovou as cinco categorias visuais, filtros de exportação, status desabilitado para modificações e limite do campo (600 aceito, 601 rejeitado); cenário removido, sem executar exportação ou persistência real.

Pendências externas: homologar sincronização Google entre dispositivos e auditoria administrativa com consentimento/permissões apropriados; identificar o incidente original de cota, não disponível nas evidências locais. Nenhuma escrita produtiva, baseline, toast ou autostart foi executada nesta retomada. CSV e JSON disponíveis; XLSX não implementado.

## Continuação — exportação XLSX e homologação

O usuário solicitou continuar a implementação. Os dois checks Windows de `93af73a` no GitHub Actions terminaram com sucesso, inclusive o [CI do PR](https://github.com/kaykyDias04/sheets_monitor/actions/runs/43426120803/job/131887832559).

XLSX implementado com os filtros e a ordenação compartilhados com CSV/JSON, células como texto literal e escrita com memória limitada. Limites do Excel são recusados sem truncamento; arquivos parciais são removidos. A biblioteca de OOXML é uma dependência explícita, com versões e avisos atualizados: 396 pacotes catalogados, sem pendências de metadados/licenças. Passaram 54 testes React focados e 14 testes Rust de gerenciamento/exportação, incluindo inspeção ZIP/XML do XLSX real. Tipos, lint, formatação, Clippy e builds frontend/Windows passaram; a compilação nativa incorporou os assets finais.

Investigação somente leitura no perfil local: esquema 5, um cadastro, uma consulta de metadados bem-sucedida e nenhum evento estrutural/registro de cota preservado. Isso não prova ausência de incidente: a tabela de consultas guarda o último resultado. Ao ser consultado sobre operações Google reais, o usuário instruiu “Ignore o problema de API e siga para as demais tasks”. A investigação da cota saiu deste ciclo; nenhuma autorização de upload no Drive, leitura do cofre ou auditoria administrativa foi concedida. Essas homologações continuam pendentes, sem tocar o banco/cofre produtivo.

Na suíte Rust completa, três testes OAuth sintéticos atingiram timeout. Duas tentativas de corrigir o mock não resolveram e foram integralmente removidas. Um probe independente reproduziu a causa no Windows: o socket aceito herdava o modo não bloqueante do listener, retornando `WouldBlock` em 0 ms apesar de timeout de 100 ms; normalizado para bloqueante, respeitou 101 ms. Essa herança é documentada em [Winsock accept](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept). A correção foi aplicada uma única vez no leitor HTTP compartilhado, mantendo listener não bloqueante, deadlines, cancelamento, PKCE e estado. O teste de fragmentação agora também cobre modo não bloqueante e rejeição prematura. A validação focal passou em 22 testes OAuth; a suíte completa final, com paralelismo padrão, passou em 300 unitários e duas integrações, com um teste de cofre ignorado. Rustfmt, Clippy e build Windows com os assets atuais também passaram após a correção. Logs em `%TEMP%/sheets-monitor-xlsx-validation/final-root-fix`. Nenhuma chamada Google ou acesso ao cofre real foi realizado.

A opção XLSX foi selecionada no componente real em Chromium, sem erros JavaScript; o cenário era explicitamente sintético e não executou exportação IPC. O HTML temporário e o servidor foram removidos/encerrados.
