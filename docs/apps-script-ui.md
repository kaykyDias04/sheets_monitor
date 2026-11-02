# Apps Script na interface

A área de scripts mostra projetos associados, autorização, última consulta, arquivos identificados, hashes e cobertura de análise. O motor pode reconhecer referências estáticas a abas, intervalos e colunas; limitações e referências dinâmicas permanecem explicitamente não verificáveis. Cobertura parcial não significa análise completa do projeto.

Não existe promessa de descobrir todos os scripts vinculados a uma planilha. Quando não houver meio autorizado de associação automática, informe o scriptId e consulte o projeto na mesma conta. O backend valida a associação; vínculo por parentId é evidência, enquanto associação manual exige intenção explícita. Marque a dependência como crítica ou opcional nas regras e salve o escopo.

Consultar scripts usa script.projects.readonly e pode depender de habilitação do acesso à API nas configurações Google Apps Script. Não executar nem editar fontes. Código e conteúdo privado não entram nos relatórios públicos; o SQLite mantém somente os dados necessários ao contrato do motor, incluindo hashes e análise, sem cópia das fontes.

Alterações de código e problemas reconhecidos são consultáveis nas atividades e diagnósticos. Nem toda alteração de código representa quebra estrutural. Veja [limitações AST](apps-script-dependencies.md) e [OAuth](oauth-setup.md).

A captura em revisão pode ser consultada na área de scripts antes de aprovar a baseline. O aviso de referência ainda não aprovada permanece visível. Se o projeto não foi observado nessa captura, a tela não reutiliza uma análise antiga como se fosse atual. Fontes não são executadas nem copiadas para relatórios públicos.
