# Primeiro acesso

Use uma instalação preparada pelo mantenedor em Windows11 x64. O candidato da Sprint06 permanece em homologação; consulte [instalação](installation.md) e [limites](known-limitations.md).

1. Abra Configurações e escolha Entrar com Google. O login e consentimento acontecem no navegador do sistema. Nunca informe senha, código de autorização ou token ao agente.
2. Revise os escopos de leitura. A descoberta Drive é opcional e requer nova autorização completa se ativada sem a concessão correspondente. Não se solicita edição ou exclusão.
3. Se faltar provisionamento, pare esse primeiro acesso e solicite ao mantenedor um build configurado. A área avançada é de desenvolvimento; o usuário final não precisa de projeto Cloud ou Client ID.
4. Cadastre um recurso autorizado por URL ou descoberta Drive. Selecione abas e configure regiões/regras. Confira a cobertura; selecionar28 abas não aprova referência das28.
5. Capture a prévia e revise pessoalmente antes de aprovar a baseline. O agente não pode aprová-la. Configuração pendente não é falha de integridade nem monitoramento completo.
6. Inicie o monitoramento na visão geral somente quando houver proteções elegíveis. Revise [cadastro e monitoramento](monitoring-guide.md).

Autostart e alertas são desligados por padrão. Habilitá-los exige sua escolha explícita e as permissões do Windows. Não enviar alerta de teste para comprovar uma configuração sem autorização específica. Desconectar é logout local; revogar a concessão é ação separada.
