# Riscos e tratamento

Os responsáveis abaixo são papéis de engenharia, não pessoas externas a quem serão enviadas mensagens. Estado inicial: aberto, salvo evidência explicitamente indicada.

| ID | Risco / consequência | Tratamento e evidência necessária | Backlog / responsável |
|---|---|---|---|
| R-01 | Protótipo pode ser confundido com entrega validada | Arquivo imutável, fonte canônica e testes da aplicação ativa separados | SM-002, SM-006 / principal |
| R-02 | Credenciais ou dados reais em Git/logs | Ignore, inspeção de padrões e revisão do diff; nunca registrar tokens | SM-007, SM-008, SM-028 / backend |
| R-03 | Quotas, planilhas grandes e intervalo inviável | Timeouts/backoff, tamanho explícito e aviso de frequência; carga de 30 arquivos | SM-022, SM-025, SM-027 / integração |
| R-04 | Edição revertida entre polls passa despercebida | Documentar limite de observação sem prometer auditoria instantânea | SM-017, SM-026 / domínio |
| R-05 | Respostas parciais ou leitura entre APIs em momentos diferentes | Snapshot completo ou falha, sem fabricar remoções; registrar incerteza | SM-009 a SM-012, SM-025 / integração |
| R-06 | Alteração de valor comum gera falso Offline | Separar evento de conteúdo da avaliação de regra/baseline | SM-014, SM-016, SM-017 / domínio |
| R-07 | Script dinâmico não verificável | Análise estática delimitada, referências desconhecidas explícitas | SM-015 / domínio |
| R-08 | Toast perdido/repetido ou consolidação apaga diferenças | Persistir eventos e entrega separadamente; testar reinício e consolidação | SM-023, SM-026 / notificações |
| R-09 | SQLite/relatórios expõem conteúdo local | Dados sintéticos agora; proteção de perfil/disco e exportação sanitizada futura | SM-005, SM-021, SM-028 / backend |
| R-10 | Migração destrutiva ou incompatibilidade futura | Transação, checksums, testes de reabertura/falha e recusa de versão futura | SM-005 / banco |
| R-11 | Fechar janela encerra processo durante monitoramento | Definir bandeja/saída e testar minimização; fundação não promete polling | SM-024 / desktop |
| R-12 | Troca de conta usa baseline de outro usuário | Separação por conta/ambiente e testes de credencial/identidade | SM-008, SM-012 / autenticação |
| R-13 | IPC/HTML de fonte vira execução ou acesso indevido | Comandos tipados, CSP e permissões mínimas; sem shell/rede arbitrária | SM-004, SM-026 / desktop |
| R-14 | Dependência muda licença ou traz vulnerabilidade | Lockfiles, inventário de licenças e auditoria antes de distribuir | SM-006, SM-028 / release |
| R-15 | Build passa, instalação/assinatura falha | Validar instalador em Windows limpo em SM-028, não inferir do probe | SM-028 / release |
| R-16 | Remoto GitHub errado ou publicação indevida | Sem origin até identificação/autorização; instrução de login sem token em texto | SM-002 / principal |
| R-17 | Dois gerenciadores alteram resolução de dependências | Manter npm lock existente nesta sprint; pnpm disponível, migração só em ADR | SM-003, SM-006 / frontend |

Aceitar um risco para planejamento não autoriza efeitos externos ou implementar a sprint seguinte. Cada fechamento deve citar comando/evidência e seu limite.

## Sprint02 — riscos e limites atuais

Cliente OAuth, APIs habilitadas, políticas Workspace e permissão adicional Apps Script dependem do projeto/conta autorizados. Não contornar bloqueios de verificação. Drive metadata opt-in é escopo restrito; revisar privacidade e exigências de verificação antes de distribuição pública. Autorização incremental não é suportada para apps instalados.

Valores/fórmulas são duas leituras sem snapshot atômico; modifiedTime não comprova mudança estrutural. Fontes Apps Script podem conter dados privados; a UI mostra texto e não persiste o código. O cofre protege credenciais no contexto Windows, sem impedir acesso por outro processo comprometido do mesmo usuário. Falhas de limpeza podem deixar chunks antigos protegidos e inacessíveis ao ponteiro ativo; não enumerar/remover credenciais alheias.

A revisão desta retomada é local pelo agente principal: os agentes anteriores esgotaram sua disponibilidade. Isso não equivale a uma auditoria independente. Mocks, build e teste real do cofre não aprovam integração Google nem distribuição pública.
