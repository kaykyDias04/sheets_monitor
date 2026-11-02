# Segurança

Não publique senhas, tokens, códigos OAuth, IDs privados ou bancos nas issues. Use somente exemplos sintéticos. A aplicação da Sprint 01 é uma demonstração local; não solicita login Google.

Se encontrar credencial em um diff, interrompa a publicação, remova-a da alteração e providencie revogação/rotação pelo titular. Não copie seu valor para logs ou comentários. Histórico com segredo exige tratamento específico; não reescreva histórico sem autorização.

Não há canal privado de reporte nem remoto GitHub configurado. Até isso ser definido pelo responsável, mantenha o achado local com descrição sanitizada, impacto e reprodução fictícia. Antes da distribuição, definir canal privado e processo de resposta em SM-028.

O backend deve controlar persistência e os futuros acessos Google. O frontend não recebe tokens nem permissões gerais de shell, rede ou filesystem. Na Sprint 01, os módulos de autenticação e Google são contratos sem implementação ativa.
