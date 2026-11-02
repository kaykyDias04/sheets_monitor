# Registro de decisões

ADRs novos usam número sequencial, contexto, decisão, consequências e estado. Decisão aceita não é evidência de funcionalidade concluída. Não reescrever registros antigos para esconder mudanças; um novo ADR pode substituí-los.

| ADR | Estado atual | Alcance |
|---|---|---|
| [0001](0001-local-first-and-boundaries.md) | Histórico do protótipo | Fronteiras úteis; parâmetros e escopo atuais vêm do anexo v1.0 |
| [0002](0002-verification-toolchain.md) | Superado operacionalmente por 0003 | Tentativa portátil GNU preservada, sem aceite do produto |
| [0003](0003-prompt01-native-environment.md) | Aceito | Ambiente Windows MSVC validado por probes isoladas |
| [0004](0004-sprint01-and-prototype.md) | Aceito | Fonte canônica, preservação e limites da Sprint 01 |
| [0005](0005-license-and-repository.md) | Aceito | Licença própria, histórico, branches e remoto local |
| [0006](0006-foundation-stack-and-verification.md) | Aceito | Stack UI, build independente e evidência nativa |
| [0007](0007-google-scopes-and-desktop-auth.md) | Aceito para Sprint 02 | OAuth desktop, escopos somente leitura e gate real separado |

Propostas futuras não autorizam sua implementação antes do prompt/sprint correspondente.

- [0008: Motor determinístico, baseline e observações parciais](0008-integrity-observation-and-approval.md).
