# Candidato OAuth interno — compilação privada

O usuário identificou o projeto Google autorizado e informou que a audiência é Interno; verificação pública é dispensada nesse cenário. O JSON Desktop foi localizado em Downloads e seu project_id coincide com o projeto informado. O conteúdo não foi registrado. Não foi necessário ler cofre, sessão, tokens ou banco de produção.

Configuração foi passada exclusivamente às variáveis Process SHEETS_MONITOR_OAUTH_CLIENT_ID e SHEETS_MONITOR_OAUTH_DESKTOP_SECRET no processo de build, sem persistir em User/Machine ou Git. Build padrão sem native-probe; escopos não ampliados, drive metadata continua opt-in. Login permanece restrito à organização Google correspondente e às políticas do administrador.

Fonte: main b0ef2c67da2786afc5ef1e9fd77bdb5735db3205, árvore af3d266974feb2015e6d53b4972adc3b07c73c2c, igual à branch integrada d7f8110. Não há alteração de runtime, dependências, migrações ou assets entre a suíte local completa e esta recompilação: somente entradas de configuração OAuth do build. Testes comportamentais locais são explicitamente reutilizados:261 unitários Rust+2 integrações+50 React, lint/fmt/types/Clippy. Build frontend foi reexecutado pelo Tauri. CI hospedado desse SHA: https://github.com/kaykyDias04/sheets_monitor/actions/runs/41756837978 . Não afirmar aprovação enquanto estiver em execução.

O candidato sem OAuth foi preservado em .tooling/release-integration/unprovisioned antes da recompilação; seu manifesto histórico em release-integration continua descrevendo aquela geração. O novo pacote terá manifesto e hashes próprios neste diretório.

Não foi aberto candidato no perfil real, aprovada baseline, enviado toast, habilitado autostart ou alterado arquivo Google. Provas instaladas/Google do novo candidato dependem de homologação humana; fixtures e confirmação de configuração incorporada não provam consentimento/login. Não é possível afirmar100% de funcionamento sem essas evidências. Preparar rascunho revisável de release, sem publicar versão definitiva ou alegar homologação concluída.

## OAuth incorporado e rascunho revisável

Compilação Tauri/NSIS concluída exit0. Presença do client ID e Desktop secret conferida em memória no binário, comparando com a configuração autorizada sem imprimir seus valores. Manifesto/hashes atuais em artifact-manifest.json e SHA256SUMS.txt. Binário/instalador sem assinatura, não executados no perfil real nesta etapa.

Rascunho1.0.0: https://github.com/kaykyDias04/sheets_monitor/releases . Três assets enviados (NSIS,checksums,manifesto), target b0ef2c6 e draft=true confirmados via API. Digest e tamanho do instalador no GitHub conferem com o arquivo local. Não existe tag pública v1.0.0 nesta verificação; nenhuma release definitiva foi publicada. A prévia0.1.0 anterior permanece intacta. Dados de API sanitizados em github-draft.json.

Login/consentimento do candidato e leitura de planilha por conta da organização foram solicitados como checkpoint humano. Não inferir conclusão pela presença de configuração no binário. Mantêm-se as pendências instaladas/carga real/24h/notificações documentadas na Sprint06. A audiência interna não exige verificação pública; não confundir essa dispensa com homologação do software.


Revisão independente confirmou hash do pacote e ausência de padrões conhecidos de credenciais nos logs/documentos (scan limitado, sem ler JSON privado). Corrigido nome público do instalador no SHA256SUMS.txt e nas notas: GitHub normaliza o espaço para ponto. Após upload corretivo, todos os três digests foram conferidos novamente. O endereço untagged de drafts pode mudar ao editar notas; a página de releases é o ponto de acesso estável do mantenedor.

## CI da fonte concluído

Workflow Windows completo aprovado no SHA fonte b0ef2c67da2786afc5ef1e9fd77bdb5735db3205: https://github.com/kaykyDias04/sheets_monitor/actions/runs/41756837978 . Snapshot sanitizado em ci-source.json. Inclui frontend, verificador, Rustfmt,testes,Clippy e build. Commits posteriores só registram docs/evidência/.gitattributes; seus pipelines foram omitidos por [skip ci], portanto não afirmar CI exato nesses heads documentais. O código/manifests/lockfiles/workflow/manual embarcado/distribution não diferem do SHA validado. Login/homologação humana do novo pacote permanecem pendentes; release segue rascunho.

## Publicação autorizada após confirmação humana

O usuário respondeu explicitamente “Login e leitura funcionaram” ao checkpoint do novo instalador. Registro limitado em human-confirmation.json; não prova todos os critérios da Sprint06.

Pré-release1.0.0 publicada com autorização original do usuário: https://github.com/kaykyDias04/sheets_monitor/releases/tag/v1.0.0 . API confirma draft=false e prerelease=true; três assets com digests locais/remotos conferidos novamente. Tag anotada v1.0.0 aponta para b0ef2c67da2786afc5ef1e9fd77bdb5735db3205, mesmo SHA aprovado no workflow completo Windows. Registro em github-published.json. Não foi marcada como latest/versão estável. A prévia0.1.0 foi preservada.

Cinco commits no range da prévia anterior até a fonte publicada; revisão pós-integração e segurança/gates independentes executados, não dispensados pelo volume. Proveniência, manifests/lockfiles, configuração de bundle, aviso de licenças e fontes de teste revisados. Versão1.0.0 é a candidata já autorizada no escopo; mudança de status de rascunho para pré-release não afirma aprovação integral. Nenhum PR/ticket remoto foi encerrado automaticamente; SM026–SM028 conservam as pendências reais do backlog.

A confirmação humana e o CI da fonte permitem disponibilizar o candidato de uso/homologação interna. Permanecem carga Google real5/15/30,ensaio24h,Windows limpo e critérios instalados/notificações ainda sem prova. Estas lacunas impedem classificar Sprint06 como APROVADA ou publicar como stable/100% comprovada. Não houve escrita Google,aprovação de baseline real,autostart ou toast pelo agente.

Artefatos gerados em target foram preservados fora do cache antes da limpeza de release: candidato OAuth em .tooling/release-artifacts/v1.0.0/Sheets.Monitor_1.0.0_x64-setup.exe; candidato anterior sem OAuth em .tooling/release-integration/unprovisioned. O manifesto conserva o caminho original do build como proveniência. Resultado da limpeza e acesso HTTP constam de cleanup.json/log e download-verification.json, após conclusão. Caches compartilhados e node_modules já existentes foram preservados.

A limpeza adicional dos temporários desta tarefa em .tooling foi rejeitada pela revisão automática por política; não foi executada nem repetida por outro caminho. Esses arquivos foram preservados. A limpeza Cargo anterior terminou com exit0 e liberou42.2GiB,com instaladores preservados e download HTTP200 confirmado.
