# Changelog

## Não publicado

- Intervalo operacional e legado ampliado para 60–600 segundos, preservando a fila e exclusão por arquivo.
- Histórico/export CSV e JSON limitados a regras protegidas; filtros combinados e export completo preservados.
- Falhas de consulta com causa, impacto e solução; diagnósticos operacionais separados das atualizações de proteção.
- Regra de proibição de exclusão passa a detectar registros sem depender da presença de fórmula.
- Contexto de auditoria separado de autoria, com fallback “Responsável não identificado”.
- Diagnóstico de cotas identifica API, operação e limite informado pelo Google; tentativas limitadas respeitam espera exponencial e `Retry-After`.
- Sincronização opcional de cadastros, abas, regras e referências na pasta privada do aplicativo no Drive, com cache e recuperação local; tokens e credenciais permanecem no dispositivo.

## 1.0.0 — candidato local em homologação,03/11/2026

Sem release pública ou distribuição a terceiros. Sprints01–05 preservadas: OAuth PKCE/cofre, leitores Google somente leitura, motor determinístico/baselines humanas, gestão/histórico, serviço Rust/outbox/bandeja e autostart opt-in.

Sprint06 prepara testes integrados e carga progressiva sintética, auditoria de segurança, avisos de terceiros e instalador NSIS x64 de produção. Resultados efetivamente executados e pendências ficam em `docs/evidence/sprint06/`. A versão do artefato não equivale a aceite da V1.0; Google real em escala, ambiente limpo e homologação humana final precisam de comprovação.

Correção de governança: resultados novos dos helpers/verificador deixam de escrever nas evidências históricas da Sprint05; conclusão da Sprint06 exige evidência e aprovação humana explícita.


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.
