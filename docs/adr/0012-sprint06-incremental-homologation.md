# ADR 0012 — homologação incremental e distribuição

Data: 03/11/2026. Estado: aceito para execução da Sprint 06, sem homologação final implícita.

O Prompt 06.1 autoriza SM-026–SM-028. O checkout recebido está limpo na base `b2c68ce`, sem artefatos ou trabalho recuperável da Sprint 06. Preservar provas anteriores e executar somente validações faltantes ou afetadas. Não reutilizar um build de desenvolvimento como produção nem uma medição sintética como vazão Google.

A campanha determinística adicional reutiliza o agendador/SQLite de produção com conta, transporte e relógio sintéticos. A cota é injetada pelo executor existente. Recuperação de processo/gaps, eventos/outbox e algoritmos de integridade conservam suas provas separadas; o teste de fila não afirma cobrir toda a cadeia de coleta HTTP ou duração de soak.

Distribuição por NSIS x64, por usuário, com autostart opt-in preservado. Avisos e manual acompanham o pacote. Inventário de licenças conservador inclui dependências de build/teste; não equivale à composição exata do binário ou auditoria jurídica. Fontes Rust sem modificações são obtidas pelos links versionados dos avisos. Lacunas de origem de avisos permanecem explícitas no inventário suplementar.

Sem alterações Google, consentimento automático, aprovação de baseline pelo agente ou publicação de release. Gates de conclusão exigem carga Google real autorizada, evidência instalada Windows e checkpoint humano quando indispensável. A ausência dessas provas mantém a sprint pendente, mesmo que testes locais ou geração do instalador passem.
