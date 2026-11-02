# Prova nativa local — Sprint 02

Executável MSVC x64 com custom-protocol e assets incorporados abriu em `http://tauri.localhost/`, com bridge Tauri ativa. Navegação nas cinco telas verificada no renderer; captura final em `native-google.png`. A imagem mostra o conteúdo WebView2, sem a moldura da janela Windows. O executável/processo/título e hash estão em `native-qa.json`.

Estado Google observado: não configurado, nenhuma conta conectada, nenhum recurso real consultado. A conexão e os testes ficam desabilitados antes da configuração. Enviar Client ID inválido retornou erro fechado de configuração, manteve geração0 e não criou configuração no banco. Fonte Unicode corrigida no rótulo de navegação após inspeção; o gate de repositório agora rejeita caracteres de substituição no código ativo.

Axe4.12.1 na área Google: zero violações e zero verificações incompletas. Sem overflow horizontal no viewport observado. Não se presume auditoria completa de todos os estados autenticados, tamanhos de tela ou leitor de tela.

Banco local existente evoluiu para versão2 com duas migrações, quinze tabelas de produto, quatro fixtures preservadas e intervalo60. Nenhum monitor real, recurso consultado ou configuração pública foi criado durante QA. Chaves estrangeiras sem erros. Somente agregados foram lidos; identidade e conteúdo não foram coletados.

O teste explícito do cofre Windows passou com bytes sintéticos em namespace UUID, substituição e exclusão. Ele não acessa credenciais de terceiros nem comprova sessão Google. Após esse teste, o build custom-protocol foi executado novamente, pois cargo test padrão pode substituir o executável pelo modo development. A tentativa inicial que dependia de localhost:1420 foi registrada como falha, não como aceite.

A instância diagnóstica foi encerrada e a porta9227 confirmada fechada. Aplicativo reaberto normalmente para o usuário configurar o cliente Desktop e consentir no navegador. Nenhum navegador de login foi automatizado. Não houve criação de projeto Cloud, habilitação de APIs ou alteração de arquivo Google pelo agente.

Homologação Google continua pendente de configuração/consentimento e leituras reais de planilha/projeto de teste. O usuário confirmou que fará essa configuração; intenção de autorizar não é prova de integração concluída. As evidências desconectadas já foram finalizadas; não capturar dados reais em screenshots públicos.
