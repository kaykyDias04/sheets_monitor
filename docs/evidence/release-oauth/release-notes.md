# Sheets Monitor 1.0.0 — pré-release para uso e homologação interna

Instalador Windows11 x64 NSIS com OAuth Desktop da aplicação incorporado. Login Google restrito aos membros da organização do projeto OAuth configurado como Interno, sujeito às políticas do administrador. Verificação pública Google é dispensada nesse cenário. Acesso Google somente leitura; baselines com aprovação humana.

O usuário confirmou login e leitura de planilha no novo candidato. Esta confirmação é humana, sem observação direta pelo agente, e cobre esses dois fluxos.

Fonte: b0ef2c67da2786afc5ef1e9fd77bdb5735db3205. CI Windows completo aprovado: https://github.com/kaykyDias04/sheets_monitor/actions/runs/41756837978 . Validação local:261 unitários Rust,2 integrações SQLite,50 React,lint,formatação,tipos,Clippy e build aprovados. Um teste de cofre é ignorado por padrão. Configuração OAuth conferida no binário sem registrar valores.

Pré-release: a Sprint06 permanece pendente de homologação integral. Carga Google real5/15/30,ensaio24h,imagem Windows limpa e demais critérios instalados/notificações documentados não foram comprovados. As campanhas sintéticas são evidências distintas. Não há afirmação de100% de funcionamento ou aprovação estável da V1.0.

Baixar Sheets.Monitor_1.0.0_x64-setup.exe e conferir SHA-256 em SHA256SUMS.txt. O instalador não é assinado. Requisitos:Windows11 x64 e WebView2; ferramentas de desenvolvimento não são requisitos de uso. Instalação em outra pasta compartilha perfil/cofre; preservar dados e usar o ambiente de homologação autorizado. Autostart e notificações dependem de preferências explícitas.

Manifesto sanitizado acompanha o pacote. Nenhum JSON OAuth privado,token,banco real ou conteúdo de planilha é distribuído como asset. Não houve escrita Google,aprovação de baseline real pelo agente,ativação de autostart ou novo toast nesta execução.
