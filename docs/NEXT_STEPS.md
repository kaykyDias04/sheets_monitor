# Continuidade e próximos passos

Fotografia feita em 03/11/2026. Esta lista registra trabalho pendente e não autoriza iniciar a sprint seguinte.

## Estado e limite de autorização

- Branch local atual: `main`; `HEAD` e `origin/main` no commit `b2c68ce0791aa8acdcc860f433dd2ed6a61849da`.
- Commits recentes: `b2c68ce` registro da publicação completa; `b8a8846` publicação autorizada; `6e9653c` correção/finalização WinRT e verificação; `23e781e` tray e controles de monitoramento; `dac71dc` serviço/outbox durável. Histórico anterior segue em `git log` e branches `codex/`.
- Sprint 05 concluída tecnicamente: SM-022 a SM-025. Leia `docs/sprint-05-report.md` e `docs/evidence/sprint05/gate3.md` antes de qualquer alteração.
- O usuário deve enviar o **Prompt 06**. Até então, apenas manutenção documental/continuidade explicitamente solicitada; não implementar SM-026–SM-028.
- `origin` está configurado e autorizado. Nenhum CI hospedado verde ou PR deve ser alegado sem consulta/evidência.

## Backlog ainda planejado

Ordem do backlog em `docs/backlog.json`:

1. **SM-026 — Testes integrados** (`SM-026.01`): organizar cenários autorizados e separar recursos Google reais de fixtures. Preservar o padrão Google somente leitura. Obter consentimento/recursos de teste apenas conforme o Prompt 06; nunca trocar dados reais por fixture na interface operacional.
2. **SM-027 — Teste de carga com 30 planilhas** (`SM-027.01`): campanha de escala ainda não realizada. Medir memória, duração e quotas com 30 arquivos sem exceder quotas, sem modificar Google e sem aprovar baseline automaticamente. Ler o prompt de Sprint 06 e fixar critérios e recursos antes de executar consultas.
3. **SM-028 — Gerar instalador** (`SM-028.01`, `SM-028.02`): revisar licenças/avisos e bundle; validar instalação/desinstalação Windows x64 e manual. Não gerar/publicar instalador até estar no escopo autorizado; identidade instalada de notificações ainda não foi homologada.

Subtarefas e títulos oficiais devem ser conferidos em `docs/backlog.json`; não renumerar IDs. Criar evidências em `docs/evidence/sprint06/` somente quando autorizado.

## Ordem recomendada após receber Prompt 06

1. Ler o novo prompt original fornecido pelo usuário, requisitos, aceite, riscos, Sprint 05 final e ADRs 0010–0011; comparar escopo com SM-026–SM-028 e registrar divergências antes de codificar.
2. Registrar contrato, gates locais/externos, limites de consentimento OAuth, dados Google permitidos e plano de evidência.
3. Reexecutar baseline local sem regravar evidências históricas. Investigar regressões antes de iniciar testes integrados/carga.
4. Homologar somente cenários autorizados; pedir intervenção apenas para credenciais/consentimento, aprovação de baseline ou efeito externo sensível indispensável.
5. Medir lote de 30 com as regras de fila (máximo global três; intervalo 60–300 segundos), cotas e segurança do usuário; fixtures não comprovam chamada Google real.
6. Fazer revisão de bundle/licenças e testes de instalação/desinstalação em escopo controlado, sem remover instalação alheia. Registrar falhas e decisões em ADR/evidências.
7. Atualizar progresso/backlog, executar gates aplicáveis e só então propor commit/PR/publicação de acordo com autorização e convenções.

## Riscos e limites conhecidos a carregar

- A tela nativa observada continha quatro planilhas cadastradas, todas em configuração pendente, e zero escopos elegíveis. As escolhas existentes foram preservadas. Requer nova revisão e aprovação humana de baseline para monitoramento real; não reabilitar automaticamente escopos antigos.
- Nenhuma escrita Google ou execução Apps Script foi realizada. Autostart permaneceu desligado. A primeira notificação de teste falhou; a segunda, via WinRT e autorizada separadamente, foi confirmada pelo usuário. O processo do probe corrigido terminou interrompido antes do relatório/callbacks finais; não alegar saída limpa/coleta completa para ele.
- O remetente do alerta em desenvolvimento era Windows PowerShell e foi informado ao usuário. Identidade final instalada e clique físico em notificação/menu de bandeja seguem sem homologação.
- Erro de notificação já corrigido: `tauri-plugin-notification` inferia identidade instalada pelo sufixo do caminho `target/debug` e podia descartar o erro assíncrono; o primeiro alerta autorizado não apareceu. A implementação passou a WinRT (`windows` crate 0.62.2), com HRESULTs observáveis e identidade explícita de desenvolvimento/release; o alerta corrigido foi confirmado pelo usuário. O getter de Setting retornou HRESULT `0x80070490` (estado desconhecido tratado de forma restrita), não prova de permissão. A execução do probe corretivo terminou `0xc000013a` antes do JSON/callback final; causa do sinal não determinada. Não repetir alerta sem nova autorização.
- Ferramenta agent-browser teve timeout de CDP inclusive em expressão simples e sessão nova; a validação de ciclo de vida usou automação nativa do Windows. Isso não prova inspeção visual final, clique físico na bandeja nem auditoria Axe.
- Não houve campanha final integrada com 30 arquivos, instalador final, instalação/desinstalação, teste de login/autostart ou CI hospedado aprovado nesta transferência.
- Falhas de um teste devem permanecer registradas; não apagar evidência anterior. Uma resposta positiva da API de notificação significa submissão, não leitura/exibição. A confirmação humana registrada é específica à tentativa.
- `archive/prototype/` não é fonte ativa. Tokens, conteúdo de planilhas, IDs privados e dumps de banco nunca devem entrar em commits/logs/evidências.

## Verificações disponíveis

Frontend (Node/npm, conforme lockfile `package-lock.json`): `npm run lint`, `npm run format:check`, `npm run typecheck`, `npm test -- --reporter=dot`, `npm run build`.

Rust/MSVC: `scripts/check-rust.ps1 -Task test`, `format-check`, `clippy` e `build`. Para novas verificações, passar `-EvidenceDirectory docs/evidence/context-transfer/rust` ou diretório da sprint corrente, em vez de sobrescrever logs históricos. `node scripts/verify-repository.mjs` verifica contratos/documentos; não substitui testes do produto.

O resumo das verificações realmente executadas nesta transferência está em `docs/evidence/context-transfer/` e será referenciado por `docs/PROGRESS.md` (nome Git rastreado como `docs/progress.md` por compatibilidade de capitalização no Windows).

## Arquivos de handoff

Para começar nova conversa Codex, carregue este documento junto com `AGENTS.md`, `docs/PROGRESS.md`, `docs/ARCHITECTURE.md`, `docs/sprint-05-report.md`, `docs/backlog.json`, `docs/risks.md`, `docs/adr/README.md`, `CONTRIBUTING.md` e o Prompt 06 que o usuário enviar. No repositório Windows, os dois caminhos em maiúsculas apontam aos arquivos históricos em minúsculas; não criar duplicatas com apenas diferença de caixa, pois NTFS e Git em Windows não as tratam de forma segura.

## Prompt de inicialização para a próxima conversa

```text
Continue o projeto Sheets Monitor neste repositório. Antes de agir, leia AGENTS.md, docs/NEXT_STEPS.md, docs/PROGRESS.md, docs/ARCHITECTURE.md, docs/sprint-05-report.md, docs/backlog.json, docs/risks.md, docs/adr/README.md e CONTRIBUTING.md. Verifique git status e HEAD para confirmar o estado real, sem descartar alterações locais. A Sprint 05 (SM-022–SM-025) está concluída; o próximo trabalho só está autorizado pelo Prompt 06, que segue anexado/enviado pelo usuário. Execute exclusivamente o escopo desse prompt. Preserve IDs, migrações e evidências históricas; não escreva em arquivos Google, não aprove baselines, não envie alertas nem habilite autostart sem autorização explícita. Separe fixtures de integração real. Não declare teste sem rodá-lo e registrar o log. Se o Prompt 06 não estiver presente, não implemente Sprint 06: solicite o prompt e limite-se a preparar a continuidade documental.
```


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.


### Atualização — audiência OAuth interna

O usuário informou que o console Google dispensa verificação porque o projeto está configurado com tipo de usuário Interno. Nesse cenário, não exigir aprovação pública como pré-condição: o OAuth limita-se aos membros da organização Google Cloud/Workspace do projeto e às políticas administrativas correspondentes. Fonte: https://support.google.com/cloud/answer/15549945#user-type-internal . A informação é relato do usuário; o nome/ID do projeto e o JSON Desktop ainda não foram fornecidos. O bloqueio atual é obter a configuração Desktop autorizada e comprovar login do candidato com conta interna elegível, além das homologações já pendentes. Não alterar a audiência para Externo nem ampliar escopos automaticamente. A futura release deve informar que o login é restrito à organização.

## Atualização vigente de continuidade

Integração original concluída em main b0ef2c6 e OAuth interno incorporado em novo pacote. O projeto e o JSON foram fornecidos e validados localmente; não repetir solicitação de credenciais. Rascunho1.0.0 disponível no GitHub, ainda não publicado definitivamente. Consultar `docs/evidence/release-oauth/report.md` e o checkpoint humano pendente antes de publicar. Não exigir verificação pública para a audiência interna; respeitar organização e políticas administrativas.
