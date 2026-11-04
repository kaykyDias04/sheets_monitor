# Sheets Monitor 1.0.0 — candidato para homologação interna

Instalador Windows11 x64 NSIS com configuração OAuth Desktop da aplicação incorporada. Login Google restrito aos membros da organização do projeto OAuth configurado como Interno; verificação pública Google é dispensada nesse cenário. Não oferece login geral para contas externas à organização. Escopos Google somente leitura; baselines continuam com aprovação humana.

Fonte: b0ef2c67da2786afc5ef1e9fd77bdb5735db3205.

Validação local:261 testes unitários Rust,2 integrações SQLite,50 React, lint,formatação,tipos,Clippy e build de produção aprovados. Um teste de cofre permanece ignorado por padrão. Cliente Desktop e configuração autorizada conferidos no binário sem registrar seus valores. CI da fonte: https://github.com/kaykyDias04/sheets_monitor/actions/runs/41756837978 .

Este rascunho não declara a V1.0 homologada. Ainda faltam login/consentimento no novo pacote, validação instalada, carga Google real e demais critérios humanos documentados. Testes sintéticos não comprovam Google real ou24h. Instalação em outra pasta compartilha perfil/cofre; usar ambiente de homologação autorizado e preservar dados. Nenhum candidato foi iniciado pelo agente no perfil real.

Pacote sem assinatura. Conferir SHA-256 em SHA256SUMS.txt antes de instalar. Instalador: Sheets.Monitor_1.0.0_x64-setup.exe. Não exige Node/Rust/MSVC para uso; requer Windows11 x64 e WebView2. Preferências de notificações e autostart permanecem opt-in.

Publicação definitiva aguarda homologação humana; não apresentar este candidato como100% funcional comprovado.
