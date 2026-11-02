# Gerenciamento de planilhas

Na área Planilhas, use a descoberta Drive ou informe uma URL HTTPS do Google Sheets. O backend valida o arquivo na conta conectada antes de registrá-lo. A descoberta pode retornar mais de 30 arquivos; somente a seleção para cadastro ocupa capacidade. O limite é transacional e duplicidades são verificadas por conta e spreadsheetId.

Até 30 cadastros presentes são permitidos, incluindo pausados. Essa decisão conservadora evita ultrapassar a capacidade ao retomar. Remover um cadastro libera a vaga; recadastrar o mesmo arquivo recupera seu registro local. Não são cadastrados automaticamente todos os resultados da busca.

Nos detalhes, atualize metadados e abas, selecione o escopo, configure proteção, capture a referência e consulte diagnóstico, scripts e atividades. Aba usa sheetId para identidade; renomear não troca essa identidade. Abas removidas permanecem identificáveis nas regras até revisão explícita. Seleção sem regras válidas mantém configuração pendente.

Pausar impede verificações operacionais sob demanda; retomar exige configuração válida e referência aprovada para o mesmo escopo. Não há execução periódica nesta sprint. Abrir a planilha usa URL Google construída pelo backend, sem aceitar destino arbitrário.

A remoção exige confirmação. Preservar histórico mantém atividades acessíveis com indicação de cadastro removido. Excluir atividades elimina esse histórico local conforme a opção apresentada; baselines aprovados são referências imutáveis e permanecem preservados. Nenhuma dessas ações exclui ou modifica o arquivo no Google.

Erros de autorização são exibidos separadamente de integridade. Uma avaliação antiga não comprova o estado atual após editar regras. Trocas de conta descartam resultados pendentes e todas as consultas são segregadas no backend.
