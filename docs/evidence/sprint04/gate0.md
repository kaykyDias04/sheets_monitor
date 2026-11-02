# Gate 0 — Sprint 04, tentativa 1 de 3

Revisão prévia do plano e dos contratos existentes. Desenho aprovado **com condições de implementação verificáveis**; nenhuma funcionalidade desta sprint é considerada pronta por esta revisão.

1. A descoberta usa o escopo existente `drive.metadata.readonly`, opt-in e concessão efetiva. Não usar `drive.file` como listagem ampla e não implementar autorização incremental incompatível com desktop. Fontes primárias em [descoberta](../../spreadsheet-discovery.md).
2. A whitelist de transporte atual aceita `/drive/v3/files/{id}`, mas ainda rejeita o caminho de listagem `/drive/v3/files`. Adicionar somente esse endpoint fixo e testar rejeição de destinos arbitrários.
3. Contrato paginado: busca literal escapada, ordenação enumerada, tamanho limitado, token opaco, unidade opcional, resultados e `incomplete_search`. Página vazia com próximo token não equivale ao fim. Troca de busca ou conta invalida o cursor. Resultado incompleto permanece visível.
4. Cadastro separado da descoberta, único por conta/arquivo, com capacidade transacional e sem seed demonstrativa na interface operacional. Remoção local confirma intenção e preservação/exclusão de histórico; nunca edita Drive.
5. Scope hash imutável da Sprint 03 identifica regras. Edição exige configuração pendente até baseline correspondente; nunca mostrar Online por padrão ou reaproveitar estado de escopo anterior como atual. Aprovação deve fixar geração, prévia, fingerprint e versão; validar revisão atual do cadastro para evitar aprovação após edição concorrente.
6. Consultas, alterações e respostas pertencem à conta/geração atual. Histórico usa filtros e paginação SQL, sem carregar todos os eventos. Retenção e exclusão não atingem dados de outra conta.
7. Configuração OAuth de distribuição pertence ao mantenedor; integração local existente não comprova publicação externa. Não inserir identificadores/segredos reais em código ou docs. Projetos Cloud e baseline de produção continuam dependendo da intervenção humana apropriada.
8. Gate 1 revisará banco/runtime/Drive; Gate 2 revisará interface e corridas de sessão; Gate 3 concluirá evidências nativas e limites. Orçamento máximo por gate: uma revisão inicial e duas revisões materiais.

Limite desta revisão: inspeção de código e documentação, sem chamadas reais ao Google, cofre de produção ou acesso a conteúdo privado. Não substitui os testes prévios exigidos nem os gates posteriores.
