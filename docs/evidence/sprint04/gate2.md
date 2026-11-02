# Gate 2 — interface operacional, tentativa 1 de 3

Inspeção especializada da interface e do leitor de sugestões de regras. Contexto aceito: Gates 0/1, contratos do motor da Sprint 03 e plano local da Sprint 04. O layout verde existente, hierarquia de cartões, formulários estruturados e comportamento responsivo são intenção visual aceita; os ajustes abaixo são de comportamento e preservam esse desenho.

## Achados materiais

**G2-1 — sincronização de sessão nas ferramentas avançadas.** A página OAuth avançada possui ciclo de sessão independente. Conectar, desconectar ou reconfigurar nela altera o backend, mas a aplicação externa mantém conta, preferências e caches anteriores até outra atualização. É necessário invalidar o estado externo ao entrar nesse fluxo e sincronizar sua sessão ao retornar ou notificar a aplicação sempre que a sessão interna mudar. Uma troca de conta não pode deixar dados anteriores apresentados como dados da nova sessão. Cobrir o percurso avançado com teste de jornada.

**G2-2 — navegação durante operação pendente.** A tabela de cadastros permite abrir uma linha enquanto outra operação mantém o lock global. `loadDetail` altera rota, seleção e epoch antes de `run` recusar o trabalho por lock; o resultado pendente é descartado e a tela termina sem os detalhes selecionados. O link de marca também contorna o bloqueio dos botões de navegação. Fazer a mesma proteção antes de alterar estado ou desabilitar os gatilhos de navegação correspondentes, com regressão de clique durante descoberta pendente.

**G2-3 — limite de arquivos aplicado às abas, identificado pelo principal.** O limite anterior de 30 itens em `MonitorScope.sheets` e na seleção operacional de abas impede selecionar todas as abas de um arquivo com 31 abas, confundindo a capacidade de 30 arquivos com a configuração interna do arquivo. O principal ajustará o limite técnico de abas para 1.024, preservando 20 mil células por captura e os limites de metadados/JSON. Testar seleção e persistência de regras válidas com 31 abas. A sugestão automática pode continuar limitada a 30 abas e 240 cabeçalhos, desde que ofereça configuração manual clara para seleções maiores; isso não pode limitar a seleção em si.

Tentativa 1 encerrada com os três achados encaminhados ao principal para remediação.

## Tentativa 2 de 3 — remediação conferida

- G2-1 resolvido: GooglePage comunica mudanças de sessão por callback estável; a aplicação principal invalida caches e fecha as ferramentas quando a geração/identidade muda. A jornada de desconexão avançada comprova a limpeza sem exigir fechamento manual.
- G2-2 resolvido: `navigate`, `loadDetail`, descoberta e consulta de eventos verificam o lock antes de alterar rota/cache; os gatilhos de abertura da tabela recebem `disabled`. A jornada com descoberta pendente confirma que clique na marca/linha não esvazia a tela e que a resposta válida continua apresentada.
- G2-3 resolvido no código: limite técnico de abas compartilhado em 1.024 para modelo, seleção e cadastro; capacidade de arquivos continua 30. Limites de células, transporte e prazo do coletor foram preservados. Regressão de seleção e persistência com 31 abas adicionada pelo principal.

**Gate 2 aprovado na tentativa 2 de 3 quanto à revisão de código, condicionado aos checks finais e à execução positiva da nova regressão Rust de 31 abas antes do encerramento.** Nenhum achado material remanescente na inspeção. Essa condição não substitui o gate nativo final.

Evidência da remediação UI: [frontend-operational-review-fixes.log](frontend-operational-review-fixes.log), 14 jornadas aprovadas. Lint e TypeScript foram informados aprovados pela lane; as execuções finais consolidadas pertencem ao principal. Não se afirma execução antecipada do teste Rust novo neste registro.

## Condições conferidas

- App operacional não importa as fixtures da fundação; serviços sem Tauri rejeitam a operação, sem simular sucesso.
- `run` confere sessão antes/depois, geração e conta do envelope; epoch descarta respostas de filtros/rotas anteriores. Fluxos normais de login/desconexão limpam os caches.
- Cadastro inicial e ausência de baseline aparecem pendentes; contagens e status vêm do backend. Seleção usa sheetId e representa abas removidas.
- Editor usa contratos do motor, sem comparação de integridade no frontend. Seleções e rascunhos não salvos bloqueiam captura/verificação. Aprovação possui checkbox explícito, fingerprint, versão esperada, revisão e geração da prévia.
- Remoção confirma cadastro local e escolha de histórico, explicitando que referências aprovadas são preservadas e que arquivos Google não são editados.
- Filtros e páginas são enviados ao backend; diagnóstico não atribui autoria nem executa correções. Preferências futuras são apresentadas como preferências ou indisponíveis.
- Sugestão de regras lê cabeçalhos delimitados, preserva regras já válidas, não adivinha fórmulas nem salva/aprova automaticamente. Conta/revisão são conferidas após a leitura; offsets e cobertura rejeitam dados incompatíveis. Cinco testes locais de proposta informados pelo principal, ainda sujeitos à suíte final.

Precisão documental: o limite de três chamadas lógicas a `structural_grid` não significa três tentativas HTTP, pois o transporte já pode repetir cada consulta. Os registros devem distinguir chamadas lógicas de tentativas HTTP.

Esta revisão é de fontes; testes finais, build e inspeção nativa serão registrados pelo principal e pelo Gate 3. Não se declara homologação Google real por componentes com mocks.

## Tentativa 3 de 3 — abertura do arquivo no navegador

Achado **G2-4**, identificado pelo principal após a tentativa anterior: `open_spreadsheet` reutilizava o abridor OAuth, cuja whitelist aceita somente o endpoint de autorização do Google. O botão de abrir planilha acabaria recusando o endereço `docs.google.com`.

Correção conferida somente quanto a esse achado e aos riscos introduzidos: o comando consulta o cadastro da conta atual, valida o identificador e constrói internamente `https://docs.google.com/spreadsheets/d/{id}/edit`. O abridor do sistema recebe apenas esse endereço canônico, dentro do guard de geração. Nenhum endereço arbitrário é aceito do frontend e o abridor OAuth permanece com sua restrição original. O ponto de injeção do abridor é privado e serve ao teste; não é comando Tauri nem configuração pública.

A nova regressão verifica o endereço canônico, a rejeição de cadastro ausente e a ausência de refresh OAuth para essa operação local. **Gate 2 aprovado na tentativa 3 de 3 quanto ao código, condicionado à execução positiva dessa regressão na suíte final.** As decisões anteriores de G2-1 a G2-3 foram preservadas e não reabertas. A suíte consolidada React atual comprova 40 testes aprovados em oito arquivos, conforme [frontend-test.log](frontend-test.log). O teste Rust de 31 abas já passou na suíte da fase 2, conforme informado pelo principal; a revisão nativa final continua pertencendo ao Gate 3.

Confirmação final da condição de Gate 2 pelo principal: [open-fix/rust-test.log](open-fix/rust-test.log) passou em 187 unitários, duas integrações e um teste de cofre ignorado por padrão, incluindo a regressão do abridor e a seleção de 31 abas. Clippy da mesma versão aprovado em open-fix/rust-clippy.log. Os 40 testes React consolidados passaram. A falha anterior de dois timeouts de testes de URL durante execução concorrente foi preservada em frontend-concurrent-timeout.log; os testes passaram a representar colagem de URL em vez de digitação caractere a caractere, preservando suas asserções funcionais. Sem achado material pendente no Gate 2.
