# Baselines pela interface

Uma baseline é a referência estrutural aprovada para uma configuração. Salvar regras não significa aprovar a estrutura. Capturar registra um snapshot e sua cobertura; a prévia permite revisar abas, regiões, cabeçalhos, fórmulas protegidas e dependências.

Para aprovar, confira o resumo e confirme explicitamente. A aprovação fixa snapshot, fingerprint, versão esperada, geração da sessão e revisão do cadastro. Alterações concorrentes ou troca de conta recusam uma prévia antiga. Se ajustar regras, salve e capture novamente antes de aprovar.

A primeira referência cria a versão inicial; mudanças legítimas exigem nova aprovação. As referências anteriores continuam disponíveis. Comparar versões é uma operação local no backend, com indicação de mudança de escopo e diferenças limitadas; não dispara avaliação remota nem redefine a referência ativa.

Nenhuma referência é atualizada automaticamente. Sem baseline correspondente às regras atuais, o cadastro fica em Configuração pendente. Uma baseline aprovada não equivale a uma verificação recente. Executar verificação sob demanda captura o escopo atual e usa o motor Rust.

Aprovação de dados reais de produção é feita pelo usuário. Fixtures podem automatizar aprovação somente em testes. Os testes reais preservam esse limite e não alteram arquivos Google.

A numeração de versão é por escopo de regras no motor existente. Ao comparar configurações diferentes, a tela distingue o escopo e a data, mesmo que ambas tenham versão 1. Somente a referência do escopo configurado atualmente aparece como ativa na visão operacional.
