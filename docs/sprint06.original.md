# PROMPT 06 — Homologação e distribuição do Sheets Monitor V1.0

Você é o agente de engenharia responsável pela Sprint 06 do projeto Sheets Monitor.

## 1. Contexto e objetivo

O Sheets Monitor é um aplicativo desktop Windows 11, desenvolvido com Tauri 2, React, TypeScript, Rust e SQLite, destinado ao monitoramento de até 30 planilhas Google Sheets.

A aplicação contempla autenticação OAuth 2.0 com PKCE, integração com Google Sheets API, Drive API e Apps Script API, análise de integridade estrutural, baseline versionado, histórico de alterações, notificações nativas e monitoramento em segundo plano.

Sua missão é executar integralmente a Sprint 06, realizando testes integrados, testes de carga, correções de defeitos, empacotamento e preparação da distribuição V1.0.

Utilize as diretrizes técnicas do repositório, o AGENTS.md, a documentação existente e as implementações reais das Sprints 01–05 como fontes de referência.

Não assuma que funcionalidades estão prontas apenas porque foram previstas no planejamento.

## 2. Autonomia operacional

Execute autonomamente todas as atividades técnicas possíveis:

1. Inspecionar o repositório e identificar a arquitetura real.
2. Auditar as funcionalidades implementadas.
3. Criar e executar testes automatizados.
4. Preparar dados e ambientes de teste.
5. Corrigir defeitos encontrados.
6. Executar testes de integração e carga.
7. Validar a segurança e a persistência.
8. Compilar a aplicação para Windows 11 x64.
9. Gerar o instalador de produção.
10. Produzir documentação e relatórios.
11. Criar commits em uma branch de trabalho.

Não solicite autorização para decisões técnicas rotineiras e reversíveis.

Solicite intervenção humana apenas para autenticação, permissões, credenciais indisponíveis, assinatura digital ou homologação que não possa ser automatizada.

Não introduza novas funcionalidades fora do escopo V1.0.

## 3. Etapa A — Auditoria inicial

Verifique o estado das entregas das Sprints 01–05.

Identifique:

- Funcionalidades concluídas e incompletas.
- Testes existentes e cobertura efetiva.
- Falhas conhecidas.
- Dependências e requisitos de compilação.
- Configurações de produção.
- Riscos de segurança.
- Pendências relacionadas às APIs Google.

Registre um relatório inicial de prontidão.

Não altere os critérios de aceite para adaptar a homologação a funcionalidades ausentes.

## 4. Etapa B — Testes integrados

Implemente e execute testes cobrindo:

- Login, logout, expiração e renovação de sessão.
- Cadastro, edição, remoção e pausa de planilhas.
- Monitoramento integral ou por abas.
- Criação e aprovação de baseline.
- Alterações de valores e linhas permitidas.
- Exclusão e reordenação de colunas protegidas.
- Alteração e quebra de fórmulas críticas.
- Exclusão de abas monitoradas.
- Alterações de código Apps Script acessível.
- Invalidação de dependências estruturais críticas.
- Classificação Online, Offline, Verificando e Indeterminado.
- Notificações nativas e histórico persistente.
- Funcionamento minimizado.
- Recuperação após falhas de rede, limites de API e reinicialização.

Use testes unitários, integração, mocks e testes reais conforme apropriado.

Diferencie explicitamente resultados simulados de resultados obtidos com serviços reais.

Não declare como aprovado qualquer teste que não tenha sido executado.

## 5. Etapa C — Segurança e integridade

Audite:

- Armazenamento seguro de tokens.
- Escopos OAuth e permissões.
- Ausência de credenciais expostas.
- Logs sem dados sensíveis.
- Tratamento de erros de API.
- Segurança dos comandos Tauri.
- Integridade das migrações SQLite.
- Persistência após reinicialização.
- Preservação do baseline sem aprovação indevida.
- Ausência de modificações não autorizadas nas planilhas e scripts.

Corrija vulnerabilidades identificadas dentro do escopo da sprint.

## 6. Etapa D — Testes de carga

Execute testes progressivos com 5, 15 e 30 planilhas.

Respeite os parâmetros:

- Até 30 arquivos cadastrados.
- Até 3 verificações paralelas.
- Intervalo inicial de 60 segundos.
- Intervalo configurável de 60 a 300 segundos.
- Histórico padrão de 90 dias.

Avalie:

- Tempo de cada ciclo.
- Latência de detecção e notificação.
- Consumo de CPU e memória.
- Crescimento do SQLite.
- Tratamento de limites e cotas.
- Duplicação indevida de eventos.
- Recuperação após falhas.
- Comportamento com alterações simultâneas.

Realize teste prolongado, com meta de 24 horas, quando houver ambiente disponível.

Caso não seja possível cumprir a frequência desejada, registre a limitação e confirme que o aplicativo informa corretamente essa condição.

## 7. Etapa E — Correções e regressão

Corrija automaticamente todos os defeitos identificados que estejam dentro do escopo aprovado.

Após cada correção relevante, execute os testes afetados e a suíte de regressão pertinente.

Não elimine testes para obter aprovação artificial.

Não reduza a cobertura nem enfraqueça regras de integridade.

Registre defeitos corrigidos e pendências remanescentes.

## 8. Etapa F — Build e instalador

Prepare a aplicação para distribuição no Windows 11 x64.

Execute:

1. Verificação de tipos.
2. Lint e formatação.
3. Testes frontend.
4. Testes Rust.
5. Testes integrados disponíveis.
6. Compilação de produção.
7. Empacotamento Tauri.
8. Geração de instalador .exe ou .msi.
9. Cálculo de SHA-256.
10. Validação da instalação e desinstalação.

Verifique o funcionamento sem ferramentas de desenvolvimento instaladas.

Não considere o build de desenvolvimento como substituto do instalador final.

Não declare instalação validada sem execução efetiva no Windows.

## 9. Etapa G — Documentação e entrega

Produza ou atualize:

- README.md
- CHANGELOG.md
- Documentação de instalação
- Guia de primeiro acesso
- Documentação de configuração OAuth de distribuição
- Guia de cadastro e monitoramento
- Limitações conhecidas
- Relatório de testes integrados
- Relatório de carga
- Relatório de segurança
- Relatório de homologação V1.0

Inclua os comandos necessários para reproduzir a compilação.

Não registre credenciais privadas na documentação.

## 10. Critérios obrigatórios de conclusão

A Sprint 06 somente poderá ser considerada aprovada quando:

1. Todos os testes críticos forem aprovados.
2. A autenticação Google funcionar no ambiente autorizado.
3. As APIs necessárias estiverem integradas e validadas.
4. A classificação de integridade estiver correta.
5. As notificações e o histórico funcionarem.
6. O monitoramento em segundo plano estiver operacional.
7. O teste de carga demonstrar estabilidade para 30 planilhas dentro das limitações documentadas.
8. Não existirem falhas críticas de segurança conhecidas sem tratamento.
9. O instalador de produção for gerado.
10. A instalação real no Windows 11 for validada.
11. A documentação estiver atualizada.
12. A homologação humana final for solicitada.

Se algum critério não puder ser comprovado, classifique a sprint como pendente ou bloqueada, indicando o motivo.

## 11. Relatório final obrigatório

Ao concluir, apresente:

- Resumo executivo da Sprint 06.
- Funcionalidades verificadas.
- Arquivos modificados.
- Testes executados e resultados reais.
- Métricas de carga.
- Defeitos encontrados e corrigidos.
- Riscos e limitações remanescentes.
- Localização dos artefatos de instalação.
- Hash SHA-256 dos instaladores.
- Pendências humanas.
- Instruções de instalação e primeiro acesso.
- Parecer técnico de prontidão da V1.0.

Classifique o resultado como:

**APROVADA:** todos os critérios técnicos foram comprovados e a homologação final foi concluída.

**PENDENTE DE HOMOLOGAÇÃO:** testes técnicos concluídos, aguardando validação humana.

**BLOQUEADA:** existem falhas críticas, testes não executados ou requisitos essenciais não comprovados.

## 12. Restrição final

Execute somente a Sprint 06.

Não inicie uma Sprint 07, V1.1 ou qualquer expansão funcional.

Não faça publicação pública, lançamento de release definitivo ou distribuição a terceiros sem autorização explícita.

Mantenha o trabalho em branch própria, respeitando as proteções do repositório e os fluxos de pull request.

Não declare sucesso sem evidências reais.

Ao finalizar a execução técnica, apresente o relatório e aguarde a homologação final do usuário.