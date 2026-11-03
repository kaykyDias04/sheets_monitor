# PROMPT 06.1 — Retomada econômica da Sprint 06

Você é o agente de engenharia responsável por retomar a Sprint 06 do projeto Sheets Monitor V1.0, interrompida anteriormente por esgotamento da quota de utilização do Codex.

## OBJETIVO

Concluir todas as atividades pendentes da Sprint 06 com o **menor consumo possível de tokens, contexto, tempo de execução e recursos computacionais**, preservando integralmente a qualidade, segurança, estabilidade e critérios de homologação da aplicação.

**PRINCÍPIO CENTRAL: NÃO REFAÇA O QUE JÁ ESTÁ CORRETO.**

Trabalhe incrementalmente sobre o estado atual do repositório. Priorize conclusão, correção e validação, não reimplementação.

## 1. RECUPERAÇÃO INTELIGENTE DO ESTADO

Antes de qualquer implementação:

1. Consulte o estado atual do Git (`git status`, `git diff --stat`, histórico recente).
2. Identifique arquivos e alterações já produzidos na Sprint 06.
3. Consulte relatórios, logs resumidos e documentação de progresso existentes.
4. Identifique testes já aprovados, falhas conhecidas e tarefas incompletas.
5. Consulte as diretrizes técnicas apenas nos trechos necessários.

Não percorra indiscriminadamente todos os arquivos do repositório.

Não repita análises completas já documentadas.

Não reverta nem sobrescreva alterações existentes sem justificativa.

Se não houver evidências suficientes de que determinada tarefa foi concluída, realize a verificação mínima necessária.

Classifique internamente cada atividade como:
- CONCLUÍDA E COMPROVADA
- PENDENTE
- FALHOU
- BLOQUEADA

Execute apenas o trabalho necessário para resolver os três últimos estados.

## 2. ESTRATÉGIA DE ECONOMIA DE RECURSOS

Adote obrigatoriamente:

### Leitura seletiva
- Utilize buscas direcionadas por arquivos, símbolos e mensagens de erro.
- Leia somente os trechos necessários.
- Evite imprimir arquivos extensos no terminal.
- Não releia documentação integral sem necessidade.

### Execução incremental
- Não recrie componentes existentes.
- Não reinstale dependências funcionais.
- Preserve caches e artefatos válidos.
- Não execute builds completos após cada alteração.
- Agrupe correções relacionadas antes de validar.
- Execute inicialmente apenas os testes afetados.

### Testes otimizados
- Reutilize evidências válidas de testes anteriores.
- Execute testes unitários direcionados durante as correções.
- Execute a suíte completa apenas quando necessária para a homologação final.
- Não repita testes aprovados sem alterações relevantes nas dependências ou no código correspondente.
- Utilize mocks para testes determinísticos, reservando APIs reais para validações que dependam delas.
- Não substitua testes reais obrigatórios por mocks.

### Controle de logs
- Utilize saída resumida por padrão.
- Em caso de erro, consulte apenas a seção relevante do log.
- Evite comandos com saída excessiva.
- Salve relatórios detalhados em arquivos locais quando necessário, sem despejar seu conteúdo integral na conversa.

### Controle de contexto
- Mantenha respostas curtas e objetivas.
- Não narre cada operação.
- Não reproduza arquivos completos na resposta.
- Não gere explicações extensas para decisões técnicas triviais.
- Registre o progresso em arquivo persistente para permitir retomadas futuras.

## 3. RETOMADA DA SPRINT 06

A Sprint 06 possui três entregas obrigatórias:

SM-026 — Testes integrados.
SM-027 — Testes de carga com até 30 planilhas.
SM-028 — Instalador Windows 11.

Verifique quais atividades já foram realizadas e quais permanecem pendentes.

Não reinicie a sprint.

### SM-026 — Testes integrados

Priorize os cenários críticos:

- Autenticação Google e renovação de sessão.
- Integração com Sheets, Drive e Apps Script.
- Cadastro e monitoramento de planilhas.
- Classificação Online, Offline, Verificando e Indeterminado.
- Detecção de alterações estruturais e fórmulas críticas.
- Baseline e persistência SQLite.
- Notificações e execução em segundo plano.
- Recuperação de falhas.
- Segurança de credenciais.

Reutilize resultados comprovados que permaneçam válidos.

Corrija somente os defeitos encontrados.

### SM-027 — Testes de carga

Valide o comportamento com até 30 planilhas, respeitando:

- Máximo de 3 verificações paralelas.
- Intervalo configurável de 60 a 300 segundos.
- Tratamento de cotas das APIs.
- Persistência de eventos.
- Recuperação de falhas.

Reutilize resultados anteriores quando houver evidências suficientes.

Não execute testes prolongados novamente se já existirem resultados válidos.

Caso um teste prolongado obrigatório ainda esteja pendente, execute-o de maneira eficiente, com logs compactos e monitoramento de métricas, sem reduzir sua duração necessária.

Diferencie testes simulados de testes reais.

### SM-028 — Distribuição

Verifique se já existem builds e instaladores válidos.

Caso existam, avalie se continuam compatíveis com o código atual.

Gere novos artefatos somente quando necessário.

Valide:
- Compilação de produção.
- Instalador Windows x64.
- Inicialização independente do ambiente de desenvolvimento.
- Persistência e funcionamento após instalação.
- Integridade dos artefatos.
- Documentação de instalação.

Não execute builds redundantes.

## 4. QUALIDADE NÃO NEGOCIÁVEL

A otimização de recursos NÃO autoriza:

- Ignorar testes críticos.
- Desabilitar verificações de segurança.
- Remover testes para eliminar falhas.
- Reduzir artificialmente a cobertura.
- Declarar sucesso sem evidências.
- Substituir integrações reais por resultados simulados na homologação.
- Ignorar erros de compilação.
- Alterar regras de integridade para facilitar testes.
- Omitir defeitos críticos.
- Reduzir o escopo aprovado da V1.0.

Otimize o processo de desenvolvimento, não os requisitos do produto.

## 5. EXECUÇÃO AUTÔNOMA

Execute todas as tarefas técnicas possíveis sem solicitar confirmações intermediárias.

Solicite intervenção humana apenas quando indispensável para autenticação, autorização, permissões administrativas, credenciais ou homologação.

Não faça perguntas sobre decisões técnicas rotineiras.

Não inicie funcionalidades da V1.1.

Não publique releases ou distribua o aplicativo a terceiros sem autorização.

Respeite as proteções de branches do repositório.

## 6. CHECKPOINT PERSISTENTE

Crie ou atualize o arquivo:

`docs/SPRINT_06_PROGRESS.md`

Mantenha nele, de maneira concisa:

- Tarefas concluídas e evidências.
- Testes aprovados e comandos utilizados.
- Pendências identificadas.
- Falhas e correções realizadas.
- Último estado válido do build.
- Próxima ação necessária.
- Bloqueios externos.

Atualize esse arquivo ao concluir cada etapa significativa.

Se a execução for interrompida novamente, o próximo agente deverá conseguir retomar a partir desse arquivo, sem reanalisar o projeto inteiro.

Não armazene segredos, tokens ou dados sensíveis nesse checkpoint.

## 7. VALIDAÇÃO FINAL EFICIENTE

Após concluir as correções:

1. Execute verificação de tipos e lint pertinentes.
2. Execute a suíte automatizada completa necessária à homologação.
3. Execute os testes de integração real ainda pendentes.
4. Confirme os resultados de carga.
5. Gere o build final uma única vez, salvo falha ou alteração posterior que o invalide.
6. Valide o instalador no Windows 11.
7. Atualize a documentação final.

Se algum teste falhar, corrija a causa e reexecute os testes afetados, seguidos das verificações de regressão necessárias.

Não repita indiscriminadamente toda a sequência.

## 8. RELATÓRIO FINAL COMPACTO

Ao terminar, apresente somente:

1. Estado da Sprint 06.
2. Atividades reaproveitadas.
3. Atividades concluídas nesta retomada.
4. Testes executados e resultados.
5. Defeitos corrigidos.
6. Instalador gerado e localização.
7. Pendências ou bloqueios.
8. Próxima ação humana, se necessária.

Evite relatórios narrativos extensos.

Classifique a sprint como:

- APROVADA
- PENDENTE DE HOMOLOGAÇÃO
- BLOQUEADA

Não declare aprovação se faltarem evidências obrigatórias.

## 9. REGRA DE EFICIÊNCIA MÁXIMA

Antes de qualquer operação potencialmente custosa, avalie internamente:

1. Esta operação é realmente necessária?
2. Já existe resultado válido equivalente?
3. Posso restringi-la aos arquivos ou testes afetados?
4. Posso evitar produzir saída extensa?
5. Esta operação contribui diretamente para concluir a Sprint 06?

Se a operação for redundante, não a execute.

Priorize ações de alto impacto e baixo custo computacional.

Não exponha esse raciocínio na resposta.

## 10. INSTRUÇÃO FINAL

**RETOME A SPRINT 06 EXATAMENTE DO ESTADO ATUAL DO REPOSITÓRIO.**

Não comece do zero.

Não reimplemente funcionalidades concluídas.

Não faça auditorias completas repetitivas.

Não sacrifique a qualidade para economizar recursos.

Execute autonomamente as pendências até concluir a sprint ou encontrar um bloqueio real.

Mantenha o checkpoint atualizado para que nenhuma interrupção futura obrigue a repetir trabalho.

Comece verificando o estado do Git e o progresso já registrado, com saída compacta, e prossiga diretamente para a primeira pendência comprovada.