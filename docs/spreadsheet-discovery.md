# Descoberta de planilhas

A Sprint 04 usa `GET https://www.googleapis.com/drive/v3/files`, sem baixar células. O filtro fixo seleciona `application/vnd.google-apps.spreadsheet` e exclui a lixeira. A busca por nome é adicionada como literal escapado; ordenação aceita somente opções conhecidas. Cada chamada retorna uma página limitada, sem confundir o limite local de 30 cadastros com o número de arquivos descobertos. A próxima página utiliza `nextPageToken`; até uma página vazia pode exigir continuação. Token rejeitado exige reiniciar a busca. A página só encerra a consulta quando não existe próximo token; isso ainda não garante completude quando `incompleteSearch` é verdadeiro. [Referência files.list](https://developers.google.com/workspace/drive/api/reference/rest/v3/files/list).

O pedido inclui `supportsAllDrives=true` e `includeItemsFromAllDrives=true`. O corpus `allDrives` pesquisa arquivos acessados pelo usuário e unidades das quais é membro, conforme suas permissões. Uma busca incompleta deve ser sinalizada e pode ser restringida a uma unidade com `corpora=drive` e `driveId`. Isso não concede acesso adicional nem promete enumerar todo arquivo de uma organização. [Suporte a unidades compartilhadas](https://developers.google.com/workspace/drive/api/guides/enable-shareddrives).

## Permissões e privacidade

O escopo já utilizado na Sprint 02, `https://www.googleapis.com/auth/drive.metadata.readonly`, permite a descoberta ampla de metadados necessária. Ele é opcional, sujeito ao consentimento e à concessão efetiva. `drive.file` restringe o acesso a arquivos compartilhados com o aplicativo e não atende a essa descoberta geral. O escopo escolhido é **restrito**; nomes e identificadores podem revelar informações privadas, mesmo sem células. Nenhuma lista real deve ser adicionada às evidências públicas. [Escopos Drive](https://developers.google.com/workspace/drive/api/guides/api-specific-auth).

Não se amplia a autorização silenciosamente. Google não suporta autorização incremental em clientes desktop; reautorizar solicita novamente o conjunto completo escolhido, com navegador do sistema, PKCE e callback loopback. O cliente OAuth pertence à aplicação e deve ser provisionado pelo mantenedor. O usuário final não deve fornecer IDs de projetos ou credenciais Cloud. Clientes desktop são públicos: um segredo incorporado não oferece confidencialidade. [OAuth desktop](https://developers.google.com/identity/protocols/oauth2/native-app).

## Publicação

Um cliente que funciona para usuários de teste não está automaticamente liberado para qualquer conta. O mantenedor precisa de projeto autorizado, APIs habilitadas, consentimento e políticas de privacidade coerentes; publicação com escopos restritos está sujeita à verificação, salvo exceções aplicáveis. Trânsito ou armazenamento dos dados restritos por servidores pode exigir avaliação de segurança. O aplicativo permanece local, sem servidor próprio obrigatório; isso não constitui aprovação do Google. [Verificação de escopos restritos](https://developers.google.com/identity/protocols/oauth2/production-readiness/restricted-scope-verification).

## Contrato de implementação e evidências

O backend deve verificar opt-in, escopo concedido e geração/conta antes de responder; manter limite de resposta, timeout e retries já existentes. Campos explícitos retornam identificador, nome, tipo, modificação, compartilhamento e unidade. A interface deve descartar resultados após troca de conta, reiniciar paginação ao mudar busca/ordem/unidade e apresentar incompletude visível. Cadastrar é escolha explícita, independente da descoberta.

Fixtures verificam parâmetros, escape de busca, páginas vazias e sucessivas, mais de 30 resultados, arquivos compartilhados, unidade específica, resposta incompleta e erros. A suíte focada `cargo test --lib google::` passou em 22 testes, incluindo oito novos testes de descoberta e um de whitelist, conforme [discovery-tests.log](evidence/sprint04/discovery-tests.log). Uma tentativa anterior durante edição concorrente da estrutura de gerenciamento falhou por arquivo ainda ausente; o registro foi preservado, sem tratá-la como aprovação.

Homologação real tem registro separado e só pode ser declarada depois de consulta autorizada pelo aplicativo. Pesquisa oficial realizada em 02/11/2026; este documento e as fixtures não são prova de integração real executada.
