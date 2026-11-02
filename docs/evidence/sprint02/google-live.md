# Homologação Google real — Sprint 02

As leituras foram realizadas pela conta conectada pelo usuário, com os recursos que ele consultou e autorizou verificar. Não foram usados adaptadores simulados nesse gate. Evidência estruturada: `google-live.json`; resultados sanitizados em `google-live-read-results.log` e `google-formula-validation.log`.

| Critério | Evidência real |
|---|---|
| Login e identidade | Configuração e conta persistidas pelo fluxo OAuth; estado connected com identidade presente |
| Cofre e recuperação | Aplicativo reiniciado; sessão recuperada sem novo consentimento, sem access token em memória no início |
| Renovação | Primeira leitura após reinício obteve access token pelo refresh protegido e UserInfo, sem novo login |
| Sheets metadados | Resposta tipada com título e 28 abas |
| Sheets valores | BatchGet UNFORMATTED_VALUE com intervalo não vazio |
| Sheets fórmulas | BatchGet FORMULA; primeiro A1:D10 não tinha fórmula, e uma consulta limitada a A1:H20 em20 abas confirmou fórmula e valor calculado correspondente |
| Drive | Nome, MIME, modifiedTime e estado trashed retornados com escopo opcional autorizado |
| Apps Script | Metadados,23 arquivos dos tipos SERVER_JS/HTML/JSON, fontes presentes e hash SHA-256 válido |
| Isolamento da conta | Respostas das consultas e estado final pertencem à mesma identidade autenticada |

IDs, nome/email da conta, nome dos recursos, valores, fórmulas e fontes não foram gravados nas evidências. Apenas indicadores e contagens foram registrados. Não houve screenshot de dados reais, execução/edição de scripts, alteração de planilhas, criação de projeto Cloud, faturamento ou revogação do consentimento.

A recuperação foi verificada no aplicativo nativo e as leituras repetidas pelo IPC de produção, com `auth/` e `google/` reais. O access token não é persistido pelo backend; observar ausência inicial e disponibilidade após a consulta comprova o caminho de renovação da sessão recuperada. Nenhum token foi lido por ferramentas de inspeção ou retornado ao renderer.

Após remover o aviso fixo de homologação pendente e o campo de engenharia real_validation do DTO, as suítes locais foram executadas novamente. A verificação final do build atualizado e o encerramento da porta diagnóstica são registrados separadamente no mesmo relatório estruturado.

Limites: o teste não revogou a autorização real nem executou logout remoto; os cenários de revogação/negação/401/403/404/429/5xx continuam cobertos pelos testes controlados. A revisão independente das lanes ficou indisponível; houve revisão pelo agente principal. O aceite não inclui monitoramento contínuo, baseline, análise semântica ou instalador.
