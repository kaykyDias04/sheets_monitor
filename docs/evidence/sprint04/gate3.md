# Gate 3 — revisão final da Sprint 04, tentativa 1 de 3

**Aprovado para encerramento técnico da Sprint 04**, com os limites de homologação real registrados abaixo. Nenhum achado material pendente nesta revisão. Não requer nova aprovação humana para concluir documentação e commit; uma nova baseline de produção continua dependendo de aprovação explícita no aplicativo.

## Evidências conferidas

- Rust: log atual do ajuste de abertura com **187 unitários e duas integrações aprovados**, um teste de cofre ignorado por padrão. As regressões de abertura canônica e de 31 abas constam como aprovadas. Clippy sem warnings; build standalone final concluído. A evidência anterior explícita do cofre foi reutilizada sem afirmar nova execução.
- Interface: **41 testes em oito arquivos aprovados**, incluindo a nova exibição da análise de scripts da captura ainda não aprovada. Logs de ESLint, TypeScript, Prettier e build Vite positivos. Os assets finais coincidem com os registrados na inspeção nativa.
- Artefato: hash SHA-256 conferido diretamente no executável atual, igual ao registro nativo `BB97F7EC22ACB4DBA0341528BE6FD96A78CE57B89114E9A0872252B236EFA962`.
- Seis rotas operacionais verificadas no renderer final pelo principal; relatório Axe sem violações nem verificações incompletas no tema automático escuro. Larguras medidas de 780 e 1.280 pixels sem overflow horizontal da página. A imagem pública foi inspecionada nesta revisão e apresenta rótulos privados omitidos; não é captura da moldura Windows.
- Migração 0004 com 24 tabelas de produto e zero erros de chave estrangeira no banco nativo. Migrações 0001–0003 e evidências das Sprints 01–03 conferidas sem alterações no diff da sprint.
- Porta diagnóstica 9234 conferida fechada nesta revisão. Fechamento/reabertura normal e limitação do probe inicial documentados separadamente. O encerramento pontual de processo órfão de diagnóstico e a falha inicial de build por lock não foram ocultados.
- Scan público de identificadores privados conhecidos com zero correspondências; seu limite de cobertura está explícito. Não foi realizado acesso ao cofre nesta revisão.

Fontes locais: [relatório final](final-report.md), [resumo dos testes](tests-summary.json), [validação local](local-validation.json), [artefato nativo](native-qa.json), [acessibilidade](native-a11y.json), [banco](native-database.json), [assets finais](final-artifact-check.log), [observações de ferramentas](native-tooling-notes.md), [privacidade](private-identifier-scan.json).

## Alterações posteriores ao Gate 2

Os ajustes CSS corrigem contraste com tokens existentes e restringem o seletor legado à tabela da fundação. Não introduzem dependências ou mudam classificação de integridade. O diagnóstico de scripts agora aceita somente a análise da captura em revisão quando essa captura existe: projeto ausente não reaproveita análise antiga. Há teste específico para esse caso, indicação visual de ausência de aprovação e reinício da paginação por projeto/hash. Nenhum novo risco material encontrado nessas mudanças; decisões aceitas nos gates anteriores não foram reabertas.

## Homologação Google: conclusão delimitada

Leituras reais comprovam sessão recuperada, duas páginas Drive, cadastro pela descoberta e URL, 28 abas retornadas, associação autorizada Apps Script com 23 arquivos e prova de vínculo, persistência, pausa/retomada e remoção/restauração local. Ainda existiam páginas Drive: não se afirma descoberta completa nem carga real de 30 arquivos.

O Online real e a repetição sem novos eventos foram comprovados somente no escopo humano anterior de **uma aba e duas células**, sem fórmulas protegidas nem scripts. Não houve nova aprovação real pelo agente. Após essa homologação, a seleção passou para 28 abas e foi preservada: o estado atual é corretamente **Configuração pendente**, conforme [configuração final](final-current-configuration.json). Não se infere proteção dessas 28 abas.

Nova aprovação visual real, histórico com eventos reais não vazios, Offline/Indeterminado por mutações Google e recuperação destrutiva continuam delimitados aos testes locais/fixtures ou pendentes da ação humana apropriada. Isso não impede concluir a implementação técnica; impede apresentar essas homologações reais como realizadas. Publicação Google externa, CI hospedado e PR remoto continuam pendentes e não foram inferidos dos testes locais. [Validação real](live-validation.json).

## Fechamento permitido

Atualizar backlog, progresso, aceite e contrato com conclusão técnica de SM-018 a SM-021 e adições SM-021A/SM-021B, preservando os IDs originais e as pendências acima; executar verificação administrativa e scan final, criar commit na branch de trabalho e confirmar Git limpo. Encerrar e aguardar Prompt 05, sem implementar scheduler, notificações nativas, bandeja, autostart ou instalador.

## Tentativa 2 de 3 — preferência padrão de retenção

O principal identificou, antes do commit final, que a preferência global de histórico era salva, mas novos escopos ainda recebiam 90 dias fixos. A correção foi revisada somente quanto a esse comportamento e aos riscos introduzidos; decisões anteriores permanecem aceitas.

O backend agora lê a preferência da conta no guard inicial de geração/revisão e a aplica apenas quando o cadastro ainda não possui escopo; a proposta é validada antes de retornar. O editor recebe o padrão da preferência e o usa somente para construir um escopo novo. Alterar seleção clona e preserva a retenção de escopos salvos e rascunhos existentes. A preferência não modifica referências aprovadas, não inicia limpeza automática e não acrescenta APIs ou migrações.

Código aprovado nesta tentativa, sem novo achado material. Regressões verificam novo escopo de 365 dias e manutenção de regra existente com 30 dias; a suíte React consolidada atual passou em **42 testes**. Confirmação da suíte Rust atualizada, do novo artefato e do probe nativo do padrão fica pendente neste registro até observação das evidências finais pelo revisor. O hash e as contagens da primeira tentativa acima representam o artefato anterior a esta correção.

## Tentativa 2 — confirmação das condições pelo principal

A revisão independente do código da política padrão foi aceita, sem novo risco material, condicionada aos testes e à prova nativa. Essas condições foram cumpridas: history-default/rust-test.log registra188 unitários e2 integrações aprovados, com 1 cofre ignorado; Clippy aprovado. Frontend-test.log registra42 testes aprovados. O build standalone atualizado passou e native-history-default.json confirma novo rascunho 365 dias, restauração do padrão 90, preservação de regras/seleções existentes e ausência de salvamento automático. As condições foram verificadas pelo principal; não se afirma nova execução independente pelo revisor após a revisão de código.

A mudança é somente o default de novas proteções, sem alteração de CSS, API ou migração. A auditoria de seis rotas permanece aplicável ao CSS/markup inalterados; verificações nativas adicionais estão em native-a11y-after-history.json. Artefato atualizado em native-qa.json. Nenhuma condição de fechamento permanece pendente. Encerrar tecnicamente e aguardar Prompt 05.
