# Sheets Monitor — contrato mestre de execução

## Autoridade e escopo

O Prompt 00 estabelece exclusivamente as regras permanentes de comportamento, autonomia, segurança e qualidade. Ele substitui a autorização anterior de implementação integral e de avanço automático entre fases.

**Não iniciar nem continuar a implementação do aplicativo no Prompt 00.** A implementação será autorizada exclusivamente por prompts posteriores, numerados de 01 a 15. Conhecer o escopo completo não autoriza implementá-lo.

Cada prompt numerado autoriza somente sua própria etapa. Não implementar funcionalidades de etapas posteriores, não antecipar tarefas por dependências futuras e não avançar automaticamente para o próximo prompt. Interfaces, contratos e estruturas podem ser preparados somente quando necessários à etapa atual, sem implementar módulos futuros.

## Autonomia durante a etapa autorizada

Realizar autonomamente as operações técnicas possíveis dentro do escopo: criar, modificar e organizar arquivos; instalar e configurar dependências permitidas; executar comandos e scripts; escrever e refatorar código; executar testes; identificar e corrigir erros; gerar documentação; realizar commits em branches de trabalho; preparar builds e instaladores quando autorizados.

Não solicitar confirmação para decisões técnicas rotineiras e reversíveis dentro da etapa autorizada.

## Encerramento obrigatório de cada etapa

1. Executar os testes pertinentes.
2. Corrigir automaticamente os erros identificados.
3. Registrar alterações e evidências verificáveis.
4. Atualizar o relatório de progresso.
5. Informar funcionalidades concluídas, testes realizados e pendências.
6. Identificar qual deverá ser o próximo prompt.
7. **Encerrar a execução e aguardar o envio do próximo prompt numerado.**

Não considerar funcionalidades concluídas sem evidências. Manter documentação, testes automatizados e registros de decisões arquiteturais. Não afirmar execução de testes sem evidência.

## Segurança e intervenção humana

Minimizar a participação humana; solicitar intervenção somente quando indispensável, especialmente para autenticação, consentimento OAuth, credenciais indisponíveis, privilégios administrativos, aprovação de baseline ou operações externas sensíveis.

Nunca contornar permissões nem expor segredos. Não alterar arquivos reais do Google sem aprovação.

## Projeto de referência

Sheets Monitor: aplicativo desktop para Windows 11, Tauri 2, React, TypeScript, Rust e SQLite, integrado às APIs Google Sheets, Drive e Apps Script. Esta descrição não constitui autorização para implementar qualquer etapa.

## Estado no Prompt 00

Contrato registrado. Trabalho iniciado sob a solicitação anterior foi interrompido; os arquivos existentes foram preservados. Não instalar dependências, inicializar projeto ou implementar funcionalidades nesta etapa. Nenhuma conclusão de integração ou testes é inferida deste registro.

**Próxima ação permitida: receber o Prompt 01 e executar apenas o escopo que ele autorizar.**
