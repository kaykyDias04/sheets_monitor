# Fronteiras de segurança — Sprint 02

Documento de implementação e verificação; não é certificado de segurança nem declaração de integração real aprovada. Consulte também [SECURITY.md](../SECURITY.md) e o [ADR de autorização](adr/0007-google-scopes-and-desktop-auth.md).

## Dados por fronteira

| Dado | Destino permitido | Fora de limites |
|---|---|---|
| Access/refresh token e client secret opcional | Backend em memória e cofre Windows conforme ciclo de sessão | React responses, SQLite, arquivos de configuração, logs e relatórios |
| Conta e escopos concedidos | UI autorizada e metadados locais necessários | Evidências públicas com identidade real |
| IDs consultados, tempos e estados | SQLite, vinculados à conta | Misturar contas ou tratá-los como monitores/baselines |
| Valores/fórmulas/fontes recebidos | Resultado manual autorizado em memória/UI | Persistência automática em logs ou evidências reais |
| Hash de código | Metadado local para futura comparação | Declarar que hash significa integridade ou aprovação |

A conta identificada tem dados pessoais, mesmo sem segredo OAuth. Minimizar o que é persistido; não carregar avatar externo automaticamente apenas para decorar a interface. A proteção do conteúdo SQLite depende do perfil Windows/disco; tokens exigem proteção específica do cofre.

## Controles a verificar

- PKCE S256, state, host/caminho do callback, parâmetros únicos, uso único, timeout absoluto e bind somente loopback.
- Browser de sistema; não capturar senha nem automatizar login/verificações de segurança do usuário.
- Endpoints HTTPS fixos, redirects desabilitados, Authorization sensível e validação de IDs; bearer nunca acompanha URL arbitrária.
- Erros públicos por categoria, sem corpo bruto do provedor, SQL, tokens ou Debug sensível.
- Refresh serializado, renovação limitada, identidade/cliente corretos, geração da sessão contra resultados atrasados após logout/troca de conta.
- Cofre sem fallback plaintext; falha de rede preserva credencial; logout não revoga remotamente; revogar exige ação explícita.
- SQL parametrizado, migração antiga imutável e consultas manuais separadas de exemplos/baselines/eventos de monitoramento.
- Preview de fonte como texto, nunca HTML executável; permissões Tauri mínimas, sem shell/HTTP/frontend filesystem genéricos.

## Evidência e limites

Testes devem usar nomes e tokens sintéticos, com valores redigidos nas falhas. A prova opcional de cofre usa entrada de teste com nome exclusivo e remove somente essa entrada. Ela não lê outras credenciais do usuário e não comprova uma sessão Google real.

Screenshots de estado desconectado podem ser versionados; screenshots de conta/arquivos reais devem ser sanitizados ou mantidos fora do Git. Nunca registrar a URL de autorização completa ou o callback com code/state. Revogação e consentimento não são inferidos por passagem de tempo.

Antes de encerrar, confrontar estes controles com código e resultados executados em `docs/evidence/sprint02/`. Homologação com usuário/recursos reais permanece um gate separado.
