Sheets Monitor — Sprint 05
PROMPT 05 · Monitoramento contínuo, notificações e execução em segundo plano
Sprint 5 de 6
SM-022 a SM-025
4–6 dias de esforço estimado

1. Objetivo da Sprint 05
A quinta sprint transformará o Sheets Monitor em um aplicativo de monitoramento contínuo, capaz de funcionar autonomamente no Windows 11, identificar alterações nas planilhas Google e notificar o usuário sem exigir que a janela principal esteja aberta.

Esta etapa utilizará as funcionalidades desenvolvidas nas Sprints 01 a 04, especialmente a autenticação Google, o motor de integridade, o banco SQLite e a interface de gerenciamento.

Ao final da Sprint 05, o aplicativo deverá:
- Monitorar automaticamente até 30 planilhas cadastradas, respeitando as cotas das APIs.
- Executar verificações periódicas a cada 60–300 segundos.
- Detectar diferenças de conteúdo, estrutura, fórmulas e projetos Apps Script autorizados.
- Enviar notificações nativas do Windows 11 para todas as alterações efetivamente detectadas.
- Permanecer funcionando quando a janela for minimizada ou fechada para a bandeja do sistema.
- Recuperar-se de falhas de internet, indisponibilidade temporária das APIs e interrupções inesperadas.
- Registrar no SQLite os resultados das verificações, eventos e notificações.
Limite de escopo: esta sprint implementará o funcionamento operacional contínuo. Os testes de carga com 30 planilhas, a homologação integrada completa e a geração do instalador definitivo permanecerão para a Sprint 06.
2. Backlog técnico
ID	Implementação	Critério de aceite
SM-022	Agendador de verificações	Monitoramento periódico automático e configurável
SM-023	Notificações nativas	Alertas do Windows para alterações e falhas relevantes
SM-024	Bandeja do sistema	Funcionamento com a janela principal oculta
SM-025	Recuperação de falhas	Retentativas, controle de cotas e recuperação segura

3. Comportamento esperado no uso diário
Sheets Monitor

Executando em segundo plano


Agendador
60–300 segundos

Google APIs
Sheets, Drive e Script



Motor de integridade
Comparação com baseline e histórico anterior

SQLite
Eventos persistidos

Windows 11
Alertas nativos



O monitoramento deverá ser executado pelo backend Rust, independentemente da renderização da interface React. Fechar a janela para a bandeja não poderá interromper as verificações.
Estados de integridade preservados
Situação	Status
Planilha verificada e íntegra	Online
Violação estrutural ou dependência crítica	Offline
Avaliação em andamento	Verificando
Internet, permissão ou API impedem a verificação	Indeterminado

Alterações permitidas de conteúdo deverão gerar notificações sem modificar indevidamente o status para Offline.
4. Detalhamento das implementações

SM-022 — Agendador automático

Implementar um serviço Rust que execute verificações periódicas sem depender da interface, respeitando o intervalo definido pelo usuário.

O agendador deverá controlar até três verificações simultâneas, evitar trabalhos duplicados, manter a fila de pendências, distribuir consultas para reduzir picos de API e permitir iniciar, pausar e retomar o monitoramento.

Intervalo inicial: 60 segundos. Faixa configurável: 60–300 segundos.




SM-023 — Notificações nativas

Integrar o sistema de notificações do Windows 11, classificando eventos por gravidade e incluindo identificação da planilha, alteração detectada e horário.

Notificações repetidas poderão ser agrupadas visualmente, mas todos os eventos distintos deverão permanecer registrados no SQLite. A falta de permissão para exibir notificações não poderá interromper o monitoramento.




SM-024 — Bandeja do sistema

Criar ícone na bandeja do Windows, com menu para abrir o aplicativo, consultar o estado do monitoramento, pausar, retomar e encerrar completamente.

O fechamento da janela deverá ocultá-la em vez de finalizar o processo, exceto quando o usuário escolher expressamente sair. Também deverá existir a opção de inicialização com o Windows.




SM-025 — Recuperação e resiliência

Implementar recuperação de falhas de conexão, timeouts, limites de requisições, autenticação expirada e interrupções do aplicativo.

Utilizar retentativas com exponential backoff, jitter e respeito aos cabeçalhos de limitação enviados pelas APIs. Falhas operacionais deverão ser diferenciadas de violações reais de integridade.



5. Exemplos de notificações esperadas
Sheets Monitor

Exemplo

Alteração detectada

Planilha: Orçamento 2026

      Aba: Despesas

      A célula F12 teve seu valor modificado. Integridade preservada.

Sheets Monitor

Exemplo

Falha crítica de integridade

Planilha: Controle Financeiro

      Uma coluna protegida foi excluída. Dependência de Apps Script potencialmente comprometida.

      Status: Offline.

Sheets Monitor

Exemplo

Monitoramento temporariamente indisponível

Não foi possível consultar a API do Google. Uma nova tentativa será realizada automaticamente.

      Status: Indeterminado.


6. Prompt 05 — Implementação pelo Codex
O prompt abaixo foi estruturado para ser executado após a conclusão da Sprint 04. Ele pressupõe que o Codex possui acesso ao repositório e às diretrizes técnicas já adotadas, sem presumir que todas as funcionalidades anteriores estejam necessariamente concluídas.
PROMPT PARA COPIAR NO CODEX
Você é o agente de engenharia responsável pela Sprint 05 do projeto Sheets Monitor.

OBJETIVO
Implementar exclusivamente a operação contínua do aplicativo desktop Windows 11: agendador, notificações nativas, bandeja do sistema e recuperação de falhas.

CONTEXTO E RESTRIÇÕES
- Preserve todas as decisões arquiteturais do repositório e do contrato mestre (Prompt 00).
- Stack: Tauri 2, Rust, React, TypeScript, SQLite, Google Sheets API v4, Drive API v3 e Apps Script API.
- Capacidade: até 30 planilhas cadastradas; verificações com intervalo configurável de 60 a 300 segundos, padrão de 60 segundos, até 3 verificações paralelas.
- Monitorar alterações observáveis de valores, linhas, estrutura, fórmulas e código/dependências Apps Script autorizados.
- Não modificar planilhas nem código remoto.
- O monitoramento deve continuar quando a janela estiver oculta na bandeja.
- Não criar servidor externo nem requerer WebView ativa para o agendamento.
- Não avançar para a Sprint 06.

AUTONOMIA
Inspecione o repositório e as implementações das Sprints 01 a 04 antes de modificar código. Execute autonomamente as tarefas, escreva testes, rode build e corrija falhas. Faça decisões técnicas reversíveis sem interromper para confirmação. Solicite intervenção somente para consentimento, autenticação, elevação de privilégios ou limitações humanas incontornáveis. Não exponha tokens ou segredos.

ETAPA 1 — AUDITORIA E CONTRATOS
Identifique as interfaces atuais de autenticação, Google APIs, persistência, baselines, motor de integridade, histórico, configurações e frontend. Reutilize os módulos existentes. Não invente APIs nem duplique componentes. Se houver lacunas das sprints anteriores que bloqueiem esta sprint, implemente apenas adaptadores mínimos seguros, documentando as dependências não resolvidas.

ETAPA 2 — SM-022: AGENDADOR
Implemente um serviço assíncrono Rust independente do frontend:
1. Carregar somente planilhas ativas do SQLite.
2. Planejar a próxima verificação por planilha usando o intervalo configurado de 60–300 segundos, padrão 60.
3. Usar limite global de três verificações concorrentes.
4. Evitar verificações sobrepostas para o mesmo arquivo.
5. Impedir duplicação de workers em múltiplas janelas ou reaberturas.
6. Oferecer início, pausa, retomada, cancelamento seguro e verificação manual.
7. Preservar o estado e a data da última verificação e próxima verificação no banco.
8. Executar o motor de integridade da Sprint 03 sem duplicar suas regras.
9. Comparar estado anterior e atual com identidade estável de eventos, preservando diferenças observadas e evitando alertar indefinidamente sobre a mesma condição não alterada.
10. Distribuir consultas com limites de paralelismo e filas.
11. Suportar suspensão/retomada do Windows e mudanças de conectividade.
12. Não iniciar monitoramento com conta desconectada ou permissões insuficientes.
13. Sinalizar excesso de tempo ou volume para planilhas que não podem cumprir a frequência selecionada.

ETAPA 3 — SM-023: NOTIFICAÇÕES
Integre a API de notificações do Tauri 2 e do Windows 11:
1. Solicitar autorização quando necessária.
2. Emitir alerta para todo evento distinto efetivamente detectado, de acordo com o escopo das planilhas.
3. Classificar alterações comuns, violações críticas, indisponibilidades operacionais e recuperação.
4. Incluir nome da planilha, aba quando aplicável, descrição e horário.
5. Persistir eventos e estados de entrega de notificações no SQLite.
6. Implementar uma fila persistente de entrega, com identificadores idempotentes.
7. Consolidar notificações repetidas visualmente sem perder registros individuais.
8. Impedir alertas repetidos ilimitados sobre a mesma condição persistente.
9. Permitir abrir o aplicativo e localizar o evento correspondente ao clicar em uma notificação, quando suportado pelo ambiente.
10. Tratar permissão negada ou notificações indisponíveis sem prejudicar o monitoramento.
11. Não registrar conteúdo sensível desnecessário em logs.
12. Diferenciar evento detectado, tentativa de entrega e confirmação possível de envio ao sistema operacional; não declarar leitura pelo usuário sem evidência.

ETAPA 4 — SM-024: BANDEJA E CICLO DE VIDA
Crie integração completa com a bandeja do Windows:
1. Ícone do aplicativo e indicação de monitoramento ativo, pausado, em falha ou indeterminado.
2. Menu de contexto: Abrir, Verificar agora, Pausar/Retomar, Configurações e Sair.
3. Ocultar janela quando o usuário clicar no X, mantendo o processo ativo.
4. Reabrir janela sem recriar serviços nem perder sessão.
5. Garantir uma única instância coordenadora do monitoramento, mesmo com tentativas de abrir múltiplas instâncias.
6. Implementar opção de inicialização automática com o Windows, desligada por padrão.
7. Implementar desligamento limpo quando o usuário escolher Sair, finalizando workers, liberando recursos e persistindo estados necessários.
8. Usar ícones e recursos compatíveis com o Windows 11.
9. Não usar fechamento forçado ou loops que impeçam a saída voluntária do aplicativo.

ETAPA 5 — SM-025: RECUPERAÇÃO
Implemente tratamento resiliente para:
- Ausência de internet.
- Erros temporários das APIs Google.
- Respostas HTTP 429 e 5xx.
- Timeouts.
- Quotas esgotadas.
- Tokens expirados ou revogados.
- Permissões removidas.
- Planilhas excluídas ou movidas.
- Suspensão e retomada do Windows.
- Encerramento inesperado do aplicativo.

Regras:
1. Aplicar exponential backoff com jitter e respeitar Retry-After quando existente.
2. Evitar tempestades de retentativas.
3. Utilizar os mecanismos seguros de renovação de token da Sprint 02.
4. Exigir novo login apenas quando a recuperação silenciosa não for possível.
5. Classificar indisponibilidade operacional como Indeterminado, não Offline.
6. Manter Offline exclusivamente quando houver violação de integridade confirmada.
7. Persistir informação suficiente para retomar as verificações após reinicialização.
8. Não afirmar que alterações intermediárias ocorridas durante períodos offline foram observadas.
9. Produzir registros operacionais úteis sem expor segredos.
10. Notificar recuperação quando um estado de falha ou indisponibilidade for resolvido.
11. Evitar modificar ou aprovar automaticamente baselines.

ETAPA 6 — BANCO DE DADOS
Evolua as migrações existentes sem perder dados:
- Persistir agendamentos, status operacionais e datas relevantes.
- Registrar eventos detectados com identidade estável.
- Persistir notificações, tentativas de entrega e agrupamentos.
- Manter histórico padrão de 90 dias com retenção configurada segundo o escopo vigente.
- Garantir transações consistentes entre detecção e criação de notificações.
- Não armazenar credenciais Google em texto puro.

ETAPA 7 — INTEGRAÇÃO COM FRONTEND
Reutilize as telas da Sprint 04 e integre:
- Estado global do monitoramento.
- Última verificação e próxima execução programada.
- Pausar/retomar e verificar agora.
- Preferências de intervalo e inicialização com Windows.
- Sinalização de falhas e tentativas de recuperação.
- Histórico de eventos e notificações.
- Atualização em tempo real usando eventos do Tauri, sem polling desnecessário no frontend.

ETAPA 8 — TESTES
Crie e execute testes unitários e de integração com APIs simuladas, incluindo:
1. Agendamento padrão de 60 segundos.
2. Alteração da frequência para 300 segundos.
3. Respeito ao limite de três verificações concorrentes.
4. Ausência de execução duplicada para o mesmo arquivo.
5. Monitoramento contínuo com janela oculta.
6. Pausa e retomada.
7. Identificação de alteração de conteúdo sem status Offline.
8. Detecção de violação crítica com status Offline.
9. Persistência exata de eventos distintos.
10. Deduplicação de notificações de condições persistentes.
11. Falha de internet com status Indeterminado.
12. Recuperação após reconexão.
13. Tratamento de 429 e Retry-After.
14. Renovação de token quando permitida.
15. Falha de permissão com necessidade de nova autorização.
16. Reinício após encerramento inesperado.
17. Saída limpa pelo menu da bandeja.
18. Migrações SQLite preservando dados existentes.
19. Nenhuma correção automática em planilhas ou scripts.
20. Detecção de código Apps Script acessível segundo as permissões existentes.

Não use esperas reais longas para testar o agendador; injete relógio e transporte simulados. Execute testes nativos do Windows onde o ambiente permitir. Não declare integração com o sistema operacional testada caso somente mocks tenham sido utilizados.

ETAPA 9 — QUALIDADE E DOCUMENTAÇÃO
Execute lint, formatação, TypeScript typecheck, testes Rust, Vitest e build Tauri. Corrija problemas encontrados.
Atualize README, AGENTS.md e documentação arquitetural.
Crie um documento docs/sprint-05-report.md contendo implementações, arquivos modificados, testes efetivamente executados, resultados, limitações e intervenções humanas indispensáveis.
Faça commits na branch de trabalho seguindo as regras existentes do repositório. Não force push nem contorne proteção de branch.

CRITÉRIOS DE ACEITE
A Sprint 05 somente estará concluída quando:
- O backend realizar verificações automáticas sem depender do frontend.
- A concorrência máxima de três for comprovada.
- A configuração de 60–300 segundos estiver funcional.
- Eventos distintos detectados forem registrados com persistência.
- Notificações Windows forem emitidas quando autorizadas.
- A bandeja funcionar e a janela puder ser ocultada sem interromper o monitoramento.
- Falhas temporárias forem recuperadas sem classificação indevida de Offline.
- Pausar, retomar, verificar agora e sair funcionarem corretamente.
- Os testes aplicáveis estiverem aprovados e o build disponível for validado.
- Não houver exposição de credenciais nem mudança automática de baseline.

RELATÓRIO FINAL
Informe:
1. Implementações concluídas, parciais e pendentes por ID SM-022 a SM-025.
2. Arquivos criados e modificados.
3. Evidências e comandos de teste.
4. Resultado real da compilação Windows.
5. Limitações conhecidas.
6. Ações humanas estritamente necessárias.
7. Prontidão para iniciar a Sprint 06.

RESTRIÇÃO FINAL
Execute exclusivamente a Sprint 05. Não inicie testes de carga finais nem geração de instalador da Sprint 06. Se uma parte essencial não puder ser concluída, descreva o impedimento concretamente, sem declarar conclusão fictícia. Ao terminar, encerre a execução.O texto integral do Prompt 05 está disponível no botão de cópia, incluindo implementação, testes, critérios de aceite e relatório final.

7. Intervenções humanas previstas
A Sprint 05 deverá exigir pouquíssimas ações manuais, pois autenticação, APIs e interface já devem estar preparadas.
Atividade	Responsável	Intervenção humana
Implementar scheduler	Codex	Nenhuma
Criar notificações Windows	Codex	Permitir notificações, se solicitado
Configurar bandeja	Codex	Nenhuma
Inicialização automática	Aplicativo	Ativar preferência, se desejado
Implementar recuperação	Codex	Nenhuma
Executar testes automatizados	Codex	Nenhuma
Testar planilhas reais	Codex + usuário	Autorizar conta e acesso, quando necessário
Validar notificações reais	Usuário	Confirmar que aparecem no Windows
Homologar a sprint	Usuário	Testar funcionamento básico

8. Checklist de homologação
Validação da Sprint 05

0/10



[ ] O aplicativo continua monitorando após fechar a janela para a bandeja.
[ ] As verificações ocorrem automaticamente no intervalo configurado.
[ ] Uma alteração comum gera notificação sem colocar a planilha em Offline.
[ ] Uma violação crítica gera alerta e status Offline.
[ ] Desconectar a internet gera Indeterminado, não Offline.
[ ] Após a reconexão, o monitoramento retoma automaticamente.
[ ] As notificações e eventos ficam registrados no histórico.
[ ] Pausar, retomar e encerrar pelo ícone da bandeja funcionam.
[ ] Os testes automatizados e a compilação foram apresentados pelo Codex.
[ ] O relatório da Sprint 05 foi produzido sem pendências críticas.
 Copiar checklist

9. Resultado esperado e preparação para a Sprint 06
A Sprint 05 é a última etapa de desenvolvimento funcional prevista no roadmap original. Ao concluí-la, o Sheets Monitor deverá estar operacional para monitoramento real, com execução contínua, tratamento de falhas e alertas.

A Sprint 06 ficará responsável por comprovar a estabilidade em escala, verificar cenários críticos integrados, realizar testes com 30 planilhas simultâneas e preparar o instalador Windows.

Uma observação importante sobre o requisito de login: o monitoramento de qualquer conta Google, sem configurar manualmente um projeto Google Cloud durante o uso diário, depende de o aplicativo já ter uma configuração OAuth distribuível válida e os escopos necessários autorizados. A Sprint 05 não elimina essa dependência nem substitui a configuração de autenticação da Sprint 02. A integração automática de Apps Script também está sujeita às permissões e aos identificadores dos projetos acessíveis.

Com isso, a Sprint 05 encerra o desenvolvimento dos serviços essenciais, deixando a Sprint 06 concentrada em testes finais, estabilidade, empacotamento e distribuição.