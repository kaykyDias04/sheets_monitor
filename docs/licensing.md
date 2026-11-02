# Licenciamento e procedência

O repositório não continha LICENSE, cabeçalhos de licença própria nem remoto de origem autorizado. O trabalho existente foi produzido nesta sessão e preservado antes da reorganização no commit `ffcaa49`; não foi encontrado código de aplicação copiado de outro repositório. O titular indicado é Kayky Dias, conforme identidade local do projeto. Essa verificação de procedência é local e não constitui prova de direitos sobre material que venha a ser adicionado depois.

Adotamos [MIT](../LICENSE) para o código e a documentação próprios, uma licença permissiva com preservação do aviso de copyright/licença. A opção foi feita dentro da autorização de configurar uma licença adequada, sem publicar o repositório. Fontes oficiais: [texto MIT](https://choosealicense.com/licenses/mit/) e [identificadores SPDX](https://spdx.org/licenses/).

Pacotes e componentes externos mantêm suas próprias licenças. O arquivo LICENSE raiz não os relicencia. Componentes shadcn/Radix, ícones, Tailwind, React, Tauri e bibliotecas Rust precisam conservar os avisos aplicáveis. Metadados `license = MIT` dos manifestos identificam somente o pacote próprio.

O [inventário de metadados](evidence/repository/license-inventory.json) registra nomes/versões/licenças disponíveis. Sua cobertura é explícita: dependências Rust de plataformas não presentes no cache podem não ter sido verificadas. Expressões MPL, Unicode e atribuição não devem ser tratadas genericamente como MIT. Não se declara uma auditoria jurídica completa nem que um bundle redistribuível já esteja homologado.

Antes de SM-028, resolver itens sem metadados, inspecionar as licenças/avisos dos artefatos efetivamente distribuídos, produzir os avisos de terceiros e confirmar a distribuição Windows/WebView2. Não redistribuir SDKs, toolchains ou credenciais no repositório.

## Dependências da Sprint02

O [inventário adicional](evidence/sprint02/dependency-licenses.json) registra metadados locais das novas dependências diretas Rust (HTTP/TLS, PKCE, URL, cofre e abertura de navegador). O Cargo.lock fixa a resolução utilizada nos testes. Esse inventário não homologa redistribuição nem substitui a revisão dos avisos transitivos em SM-028. npm audit de produção tem relatório separado e não cobre advisories Rust ou todas as dependências de desenvolvimento.

## Novas dependências da Sprint 03

Tree-sitter e a grammar JavaScript foram escolhidos a partir das APIs e metadados oficiais. O [inventário adicional](evidence/sprint03/dependency-licenses.json) registra versões/licenças do cache usado na compilação. A resolução está fixada no Cargo.lock; o inventário não substitui revisão dos avisos de distribuição em SM-028.

Sprint 04 não adicionou dependências de produto. O inventário revisado da Sprint03 permanece aplicável aos mesmos pacotes e respectivas licenças; ferramentas locais de QA não integram o bundle. Distribuição final e avisos completos continuam sujeitos à etapa autorizada de instalador.
