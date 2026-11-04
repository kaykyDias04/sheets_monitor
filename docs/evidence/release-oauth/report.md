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
