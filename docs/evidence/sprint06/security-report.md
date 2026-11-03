# Sprint 06 — auditoria de segurança e integridade

Data: 03/11/2026. Escopo: código ativo Windows x64 das Sprints 01–05 na base `b2c68ce0791aa8acdcc860f433dd2ed6a61849da`, estado inicial com alterações documentais preexistentes preservadas; branch `codex/sprint-06-homologation`. Este relatório cobre a auditoria inicial, antes de eventuais correções e do instalador da Sprint 06. Não é certificado de segurança, parecer jurídico nem aprovação de distribuição.

## Modelo de ameaça

Ativos: access/refresh tokens, configuração OAuth, identidade e metadados privados, SQLite com fórmulas/cabeçalhos protegidos, referências aprovadas, eventos/outbox, disponibilidade e registro de inicialização. Entradas: IPC do WebView local, URLs/IDs, respostas HTTPS Google, callback OAuth loopback, dados SQLite, callbacks WinRT e dependências/build/CI. Atores: usuário local autenticado, conteúdo Google não confiável, serviço remoto comprometido ou resposta malformada, contribuinte/dependência maliciosos. Processo de outro usuário com controle integral do perfil Windows está fora da promessa do cofre; o produto não protege um perfil já comprometido.

Não houve acesso a credenciais reais, leitura de banco real, mutação Google, execução Apps Script, consentimento, revogação, toast, alteração de autostart ou publicação nesta lane. VM limpa, pentest remoto e validação dinâmica do instalador não foram executados nesta auditoria inicial.

## Evidências automatizadas executadas

| Comando/método | Resultado | Evidência e limite |
|---|---|---|
| `security-surface.sh .` da skill, via Git Bash | Exit 0; inventário de superfícies | `security-surface-summary.json`; somente contagens retidas, candidatos brutos ficaram em memória; inventário não demonstra vulnerabilidade |
| `git status --short --branch`, `git diff --check`, `git submodule status`, `git ls-files -s` | Branch correta, diff sem erro de whitespace, sem submódulos e sem modos executáveis/symlink rastreados detectados | Saída da inspeção; avisos Git de LF/CRLF são distintos de falha de diff |
| Scan de formas conhecidas em texto rastreado/não ignorado e histórico alcançável | 603 arquivos de texto, 15 commits, zero candidatos nas seis regras | `secret-shape-scan.json`; não é detector de entropia, não examina ignorados/binários, não prova ausência universal de segredos. Primeira tentativa falhou por ausência de `-e`, rerun corrigido concluído |
| `npm audit --omit=dev --json` | Exit 0, zero advisories de produção reportados | `npm-audit-production.json`; não cobre toda a cadeia de desenvolvimento |
| API oficial OSV `POST /v1/querybatch`, só nomes/versões públicos Cargo.lock | 512 pacotes consultados, zero erro de consulta; cinco pacotes sinalizados | `rust-osv-audit.json`; hash lock registrado, sem credenciais transmitidas |
| API OSV `GET /v1/vulns/<RUSTSEC-ID>` | Detalhes oficiais obtidos para triagem | `rust-advisory-details.json` |
| `cargo tree --manifest-path src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc -i <pacote>` para os cinco | Exit 0, `nothing to print` para todos | `rust-advisory-windows-target.log`; ausência no grafo Windows não remove advisories do lockfile/outros alvos |
| `cargo audit --version` | Ferramenta não instalada | Nenhum cargo-audit alegado; consulta OSV é alternativa limitada |
| Verificação de `gh.exe` no PATH e caminho padrão GitHub CLI | Ausente | Alertas GitHub Secret Scanning/Dependabot/CodeQL não consultados; sem alegação de estado remoto limpo |

O scan de formas procurou apenas PEM private-key, prefixos Google access/refresh/desktop secret, GitHub token e AWS access ID com comprimento mínimo. Nenhum valor correspondente foi impresso ou persistido. A enumeração do histórico utilizou `git rev-list --all` e `git grep -I -l -E -e <padrão> <commit> --`.

## Triagem de dependências

- `glib 0.18.5`: RUSTSEC-2024-0429 / GHSA-wrw7-89jp-8q8g relata unsoundness em `VariantStrIter`. **Ausente no grafo Windows x64 inspecionado**; não é vulnerabilidade de execução comprovada deste artefato Windows. Continua risco real se o projeto vier a suportar o alvo que compila essa dependência; esse alvo não está autorizado nesta sprint.
- `proc-macro-error 1.0.4`: RUSTSEC-2024-0370, sem manutenção. Ausente no grafo Windows inspecionado.
- `zbus_names 4.3.5`, `zvariant 5.15.1`, `zvariant_derive 5.15.1`: RUSTSEC-2026-0371/0370/0372, sem manutenção como crates separadas após incorporação a zbus 6. Ausentes no grafo Windows inspecionado.

Não há supressão `--ignore` adicionada nem atualização invasiva de alvos não distribuídos para fabricar scan sem alertas. A triagem deve ser reexecutada se Cargo.lock/features/alvo mudarem. Fontes primárias estão nos campos references do JSON OSV e em [RustSec glib](https://rustsec.org/advisories/RUSTSEC-2024-0429.html).

## Fronteiras revisadas sem achado substanciado

| Fronteira | Código/controle efetivamente inspecionado | Conclusão delimitada |
|---|---|---|
| OAuth/callback | `auth/protocol.rs`: OsRng 32 bytes, S256, bind IPv4 loopback/porta dinâmica, state constante, host/path/parâmetros únicos, prazo absoluto, resposta no-store | Sem caminho substanciado de callback arbitrário ou replay nesta inspeção; testes existentes cobrem casos host/state/path/duplicação/deadline |
| Sessão/cofre | `auth/manager.rs`, `auth/vault.rs`: tokens não serializam no DTO, Zeroizing/Drop, cofre keyring Windows sem fallback texto, revisão/chunks/hash e ponteiro atual; geração guardada em refresh/logout/commit | Cofre não impede processo comprometido no mesmo perfil; chunks protegidos antigos podem sobrar em falha de limpeza. Não foram lidas credenciais de produção |
| Rede/SSRF/escopos | `google/transport.rs`, `google/mod.rs`, `auth/transport.rs`: HTTPS, hosts/paths fixos, redirects none, IDs estritos, HeaderValue sensível, limites 8MiB/64KiB, timeouts e retry finito | Bearer não pode ser redirecionado a URL arbitrária pelo IPC; Google de produto é GET, POST só token/revogação OAuth explícita |
| Conta/revisão/autoridade | `integration.rs`, `management/runtime.rs`, `integrity/runtime.rs`: conta derivada da sessão, geração esperada, revisão/fingerprint e resposta sob guard | Requisições antigas/troca de conta não ganham autoridade por assunto fornecido pelo renderer |
| Baseline | `database/integrity_store.rs`: confirmação, origem permitida, captura completa, validação, transação Immediate, versão CAS; runtime confere fingerprint/conta/geração/revisão | Origem synthetic:test só admitida por cfg(test)/feature opt-in; não foi chamada aprovação real pelo agente |
| SQLite/migrações | `database/migrations.rs`, `database/mod.rs` e stores: SQL parametrizado, ledger/checksum normalizado LF, recusa versão futura e rollback transacional | Migrações históricas não alteradas pela lane. Integridade de schema não equivale a criptografia do conteúdo local |
| IPC/WebView | `app.rs`, capabilities/default.json, tauri.conf.json, serviços TS: comandos tipados, core:default, CSP sem scripts externos, sem shell/HTTP/filesystem genéricos; ausência de dangerouslySetInnerHTML | Nenhum sink substanciado de HTML/SQL/caminho arbitrário pela entrada Google inspecionada |
| Fila/cancelamento | `monitoring/work.rs`, `monitoring/service.rs`, `database/monitoring_store.rs`: limite três compartilhado, um por arquivo, geração/lease CAS, deadlines, Retry-After, retenção | Código dá proteções relevantes; campanha final de carga e estabilidade prolongada ainda necessária |
| Outbox/notificação | `notifications/service.rs`, `windows.rs`: tentativa persistida, geração durante submissão curta, recibos enfileirados sem reentrância, CAS, supressão/TTL, callbacks limitados, XML via text nodes | Sem markup/comando vindo do conteúdo; submissão não prova exibição. Crash pode causar repetição física; notificação não é exatamente-uma-exibição |
| Autostart/ciclo de vida | `desktop.rs`: HKCU apenas, comando fixo e entre aspas, conflito preservado, opt-in explícito; instância única antes de serviços | Nenhuma habilitação automática identificada; clique/login real seguem homologação própria |
| Build/CI | package.json, vite.config.ts, test-setup.ts, Cargo.toml/build.rs, workflow: sem hooks npm próprios, build Tauri e manifesto do probe, contents:read, sem secrets/deploy/pull_request_target | Actions por tag e advisories dinâmicos continuam risco da cadeia; nenhum resultado de CI hospedado foi inferido |

## Candidatos rejeitados e fatos não equivalentes a vulnerabilidade

- `open::that` seria navegador arbitrário: transporte OAuth valida accounts.google.com e runtime de planilha gera URL canônica docs.google.com com ID validado; não aceita comando/scheme arbitrário.
- `unsafe` no Windows indicaria código oculto: blocos inspecionados gerenciam inicialização/finalização COM/WinRT; não demonstram execução de conteúdo Google.
- Advisory glib como falha crítica do instalador Windows: cargo tree de alvo refutou presença no grafo do candidato inicial; não classificar o lockfile multi-alvo inteiro como superfície distribuída.
- `synthetic:test` como bypass IPC de aprovação: origem é interna e guardada por cfg; IPC não recebe origem arbitrária.
- Desktop client secret incorporado seria segredo confidencial de servidor: cliente OAuth instalado é público e requer PKCE; incorporar refresh/access token continuaria proibido. Provisão/distribuição OAuth não está homologada por essa observação.

## Needs validation e riscos remanescentes

1. Reavaliar diff final e hashes do artefato após empacotamento; esta auditoria inicial não certifica arquivos ainda não produzidos. Avisos/licenças, assinatura e identidade instalada dependem da fase F.
2. Alertas hospedados GitHub não consultados por ferramenta indisponível; estado remoto não é presumido limpo.
3. OSV consulta advisory conhecido; não cobre novos defeitos, toda a cadeia de build, conteúdo de binários terceiros ou comportamento de pacote comprometido.
4. Isolar perfil/cofre ao iniciar candidato instalado: instalação em pasta diferente não muda namespace produção nem app_data_dir. Máquina Windows limpa/usuário de teste é necessário para prova independente de estado real.
5. Homologação final Google, notificações visíveis com identidade instalada, meta 24h, carga real/quotas e máquina sem dev tools ainda não executadas nesta lane. Fixtures não substituem esses fatos.
6. Snapshot entre APIs não é transação global; polling perde mudanças revertidas entre coletas; hashes não criptografam os dados; SQLite depende da proteção do perfil/disco. Limites do motor/AST e retenção permanecem documentados.

**Resultado da auditoria inicial:** nenhum achado de exploração substanciado no escopo estático Windows x64 inspecionado. Há advisories no lockfile fora do grafo alvo e lacunas de validação explícitas. Não declarar o produto universalmente seguro nem aprovar a V1.0 a partir deste relatório.
