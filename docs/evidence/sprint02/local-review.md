# Revisão local da Sprint 02

Revisão pelo agente principal após a indisponibilidade das três lanes anteriores. Não representa gate independente Oracle nem auditoria externa. Escopo: autenticação, fronteiras IPC, leitores GET, migração2 e UI manual.

| Risco | Verificação / resultado local |
|---|---|
| Token/secret exposto por IPC ou Debug | DTOs não contêm tokens; structs sensíveis têm Debug redigido; testes de sentinelas e scan de padrões |
| Bearer enviado a URL arbitrária | IDs validados, endpoints HTTPS fixos, allowlist de caminhos e redirects desabilitados |
| Callback forjado | PKCE S256, state aleatório, host/path/GET/parâmetros únicos, bind IPv4 loopback e deadline absoluto |
| Sessão restaurada após logout durante refresh | Teste concorrente com barreiras confirma descarte antes de persistência |
| Troca de conta ou perda de escopo | UserInfo valida sujeito, concessões efetivas restringem leituras e UI descarta resultado de outra conta |
| 401 ou rede causando loop | Uma renovação após401 no runtime; GETs transitórios até3 tentativas e prazo finito |
| Permissão Drive excessiva | Opt-in drive.metadata.readonly; sem escrita; privacidade/verificação documentadas no ADR0007 |
| Cofre indisponível/corrompido | Falha fechada, sem plaintext; logout explícito pode resetar ponteiro corrompido; teste Windows isolado aprovado |
| Mistura entre contas no banco | Recursos/consultas/associações vinculados à conta, SQL parametrizado e testes de associação cruzada rejeitada |
| Mudança de schema antigo | Migração0001 intacta; upgrade1→2, rollback/ledger e preferências testados |
| Conteúdo remoto executado | Fontes mostradas como texto React; sem execução/edição ou comandos genéricos de shell |
| Mocks apresentados como Google | Browser informa ausência de backend; comandos reais sem fallback; tarefas principais bloqueadas por homologação |

Correções da revisão: associação explícita consulta sucesso de metadados Sheets, sem depender do último status da leitura de valores; logout permanece disponível quando o cofre falha ao recuperar; ponteiro de cofre corrompido permite reset sem enumerar credenciais; erros de sessão atualizam estado na UI. Clippy corrigido sem supressões.

Limites: ausência de projeto/cliente Google autorizado impede prova de consentimento, refresh real e dados das três APIs. SQLite não é criptografado e contém metadados pessoais. Troca/renovação OAuth não é repetida automaticamente. Várias chamadas Google não formam transação atômica. Chunks antigos protegidos podem ficar órfãos quando a limpeza falhar; nada remove credenciais de terceiros. npm audit de produção não substitui auditoria de supply chain Rust. Distribuição pública/instalador aguardam SM-028.
