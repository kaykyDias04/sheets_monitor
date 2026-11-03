# ADR 0011 — entrega Windows com erros observáveis

Estado: aceito na correção da Sprint 05. Complementa o ADR 0010 e substitui sua escolha do plugin para entrega Windows; preserva consentimento, outbox, retenção e separação entre submissão/exibição/leitura.

O único alerta sintético inicialmente autorizado foi submetido ao plugin 2.8.0, mas o usuário informou que não apareceu. A evidência negativa está em `docs/evidence/sprint05/native-toast-observation.json`. A fonte do plugin identifica instalação pelo sufixo do diretório: executáveis em `target/debug/examples` ou `target/x86_64-pc-windows-msvc/debug` recebem uma identidade de aplicação instalada, embora não tenham sido instalados. O erro real do envio ocorre em outra thread e pode ser descartado. Não atribuímos a falha a Não incomodar sem evidência.

O adaptador Windows passa a usar a crate oficial Microsoft `windows` e WinRT. Identidade é uma política explícita do build: desenvolvimento usa o host Windows PowerShell conhecido, com identificação de desenvolvimento; release usa somente a identidade configurada. Não se tenta outra identidade após bloqueio por usuário, aplicação ou política. Não há criação automática de atalhos, registro de identidade ou alteração de configuração Windows.

Consulta de disponibilidade distingue criação do notifier de leitura de `Setting`, com códigos HRESULT sanitizados. Um erro desconhecido não é evidência de permissão negada nem de disponibilidade. `Show` retorna seu erro diretamente. `Failed` posterior gera um recibo para a tentativa exata; os callbacks apenas enfileiram, evitando reentrar no mutex de autenticação durante `Show`. O dispatcher persiste o resultado com CAS de conta/geração/tentativa. Supressão e expiração continuam terminais; submissão anterior continua registrada como fato, sem se tornar confirmação de exibição.

Delegates possuem limite de quantidade e tempo, sem criar uma thread por alerta. Conteúdo vira nós de texto XML, sem interpretar fórmulas, nomes ou diagnósticos como comandos ou markup. Callback de ativação carrega somente ID opaco e a leitura do evento é revalidada na sessão atual.

O plugin anterior de notificações é removido do caminho de produção e das dependências; instância única permanece em Tauri. A API nativa não requer servidor. Identidade própria instalada e validação do instalador continuam na Sprint 06. Outro alerta de teste exige nova autorização, pois a autorização anterior foi consumida. Até uma observação humana positiva, a exibição permanece pendente.
