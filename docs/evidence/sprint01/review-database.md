# Gate 1 — revisão independente da persistência

Revisor: responsável pela fundação nativa, sem autoria do módulo `database` nem
da migração SQL. Tentativa 1 de 3. A revisão é estática; os resultados de execução
dos testes pertencem aos logs Cargo e não são presumidos neste relatório.

## Achado corrigido

P2: as referências de `integrity_rules.baseline_id` e
`change_events.monitoring_check_id` inicialmente aceitavam pais pertencentes a
outra planilha. O autor adicionou chaves únicas compostas nos pais e foreign keys
compostas com `spreadsheet_id`, além de dois inserts de regressão que devem ser
recusados. A correção foi relida e mantém associações opcionais nulas permitidas.

## Verificações

- A migração completa e o ledger são aplicados em uma única transação IMMEDIATE.
  Falha desfaz schema novo, ledger e `user_version` juntos.
- O runner rejeita versões futuras, sequência inválida, lacunas no ledger,
  divergência entre `user_version` e ledger e checksum de migração diferente.
- Foreign keys ficam habilitadas por conexão. Dados são parametrizados nas APIs
  expostas; os nomes interpolados em testes vêm de uma lista fechada de tabelas.
- Configurações são validadas tanto no domínio quanto no schema: intervalo
  60–300, histórico de 90 dias e inicialização automática desativada.
- As onze entidades previstas existem. Inserção/restauração além de trinta
  arquivos ativos é recusada por triggers independentes da interface.
- Fixtures são inseridas uma vez em transação e explicitamente marcadas como
  demonstração. Não se inventa aprovação de baseline, autorização de script ou
  envio real de notificação. O estado Verificando é um exemplo, sem agendador.
- O caminho do banco é fornecido pelo backend, exige caminho absoluto e rejeita
  o arquivo final quando é um symlink; o frontend não fornece caminhos de disco.

## Condições e limites

Os hashes de migração dependem dos bytes SQL. Foi recomendado ao agente principal
fixar EOL LF para migrações no Git, permitindo o mesmo checksum em builds locais
e CI. O banco da fundação é `foundation-v1.db`, separado do protótipo arquivado.
Testes de persistência não comprovam integração Google, monitoramento, integridade
real ou comportamento de um instalador, todos fora desta sprint.

Após a correção das referências cruzadas, nenhum bloqueio material permaneceu na
revisão estática. O gate de execução ainda exige os testes registrados pelo Cargo.
