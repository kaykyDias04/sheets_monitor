# Sheets Monitor V1 — requisitos provisórios

Fonte: contrato fornecido na conversa em 01/11/2026. A pasta estava vazia; o documento original mencionado pelo usuário não estava disponível. Este arquivo registra o escopo implementável, sem alegar equivalência com um documento não recebido.

## Produto

- Aplicativo Windows 11 local, Tauri 2, React, TypeScript, Rust e SQLite. Não exige servidor hospedado.
- Até 30 planilhas por ambiente, IDs explícitos. Leitura de Sheets (incluindo grid), metadados/permissões Drive e projetos Apps Script explicitamente autorizados.
- OAuth desktop no navegador do sistema, PKCE S256, state e callback loopback efêmero. Tokens restritos ao backend e ao cofre do Windows.
- Baseline inicial exige prévia e aprovação explícita do usuário. Nenhuma baseline real é aprovada pelo agente.
- Comparar todos os campos retornados pelas APIs, inclusive valores, fórmulas, formatação, estrutura, metadados, permissões e fontes/manifestos de scripts. Cada diferença detectada gera evento e notificação durável.
- Análise estática de scripts desde V1: achados são heurísticos, sem executar código e sem afirmar auditoria completa.
- Agendamento local, coleta manual, histórico, recuperação de erros, fila de notificações e relatório local.
- Falhas parciais de API não devem substituir snapshots válidos. Reinícios preservam histórico e notificações. Modo de fixtures isolado de dados reais.
- Nenhuma escrita nos arquivos Google. Nenhuma execução/deploy de Apps Script. Revogação remota depende de ação explícita, não da rotina de monitoramento.

## Limites verificáveis

Polling observa diferenças entre snapshots completos bem sucedidos; não captura alterações transitórias entre coletas. Escopos e APIs limitam o que é observável. Apps Script exige IDs e acesso explicitamente concedidos; não existe promessa de descoberta automática de todos os scripts vinculados. Dados fora da resposta ou inacessíveis não são declarados monitorados.

O aplicativo precisa permanecer em execução para agendar coletas. Notificações do Windows dependem das configurações do sistema; a fila persistente e a central do aplicativo preservam cada evento independentemente da apresentação do toast.

## Gates

1. Domínio/SQLite: limite 30, diffs, aprovação, transações, reinício, outbox e falhas testados com fixtures.
2. Adaptadores: PKCE/state/callback, leitura, falhas/retries e proteção de tokens verificados localmente. Integração Google real permanece pendente sem credenciais/consentimento.
3. Interface/Tauri: TypeScript, testes de fluxos, build web, testes Rust e build nativo conforme toolchain disponível. Evidências em `docs/evidence/`.
