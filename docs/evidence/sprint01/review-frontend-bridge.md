# Gate 2 — revisão independente do frontend e bridge

Revisor: responsável pelo backend nativo, sem autoria do frontend. Tentativa 1 de
3. Esta revisão é estática; testes e inspeção visual têm evidências próprias.

## Contrato e escopo

- `foundation_get_dashboard` e `foundation_save_settings` são os únicos comandos
  chamados pelo bridge e registrados pelo backend.
- Dashboard contém planilhas, atividades, configurações, versão do banco e
  marcação explícita `is_demo: true`. O frontend recusa respostas que não tenham
  a marcação demonstrativa no dashboard e nos itens.
- Configurações compartilham intervalo 60–300, histórico 90 e autostart falso.
  O frontend valida limites; o domínio e o schema persistente validam novamente.
- A prévia web mantém fixtures em memória. A interface distingue isso do SQLite
  no desktop e não declara persistência em disco no navegador.
- As quatro telas e os quatro estados previstos são navegáveis com dados
  demonstrativos. A UI distingue mudança comum de falha crítica nos exemplos.
- O diálogo de baseline é didático: não existe botão de aprovação nem comando
  nativo capaz de aprovar uma baseline. Login, cadastro Google e autostart não
  são ativados por controles desta sprint.
- As categorias do seed SQLite foram alinhadas ao filtro de histórico pelo autor
  do banco. A versão do banco ausente no DTO nativo inicial foi adicionada após
  a revisão cruzada do autor do banco.

## Fronteiras observadas

Não foram encontrados fetches, scripts remotos, fontes externas ou HTML injetado
no frontend ativo. Strings de dados são renderizadas por React. Diálogos usam
componentes Radix e os testes cobrem navegação, restauração de foco, preferências
e rejeição de dados não demonstrativos por um mock de IPC. O mock não substitui
abrir e operar a janela real no Windows.

## Correção observada nos testes

P2: o diálogo de baseline controlado por estado tinha vários botões de abertura,
sem um único `DialogTrigger`. Ao fechar, o foco não retornava ao botão de origem.
O autor corrigiu o componente compartilhado para capturar o elemento ativo antes
do autofocus e restaurá-lo ao fechar, respeitando handlers que previnam o evento
e verificando se o elemento continua conectado. A correção foi relida e o teste
de foco foi preservado. O shim de `ResizeObserver` foi limitado ao ambiente
jsdom; ele não altera o comportamento da aplicação no Windows.

Nenhum bloqueio material permanece nesta revisão estática. A aprovação de execução
depende dos logs de testes e build e da inspeção da janela nativa em execução.
