# Limitações conhecidas da V1.0 candidata

- O candidato local da Sprint06 não representa V1.0 homologada. Carga Google5/15/30, quotas reais, execução instalada em Windows limpo e ensaio24h dependem de ambiente/recursos ainda indisponíveis. Relatórios sintéticos não substituem esses critérios.
- Primeiro acesso depende do cliente OAuth da aplicação provisionado pelo mantenedor e das políticas Google/Workspace. Consentimento local histórico não aprova publicação externa.
- Polling não observa toda edição intermediária nem uma alteração revertida entre coletas. Sheets/Drive/Apps Script não fornecem uma transação global da captura.
- Online refere-se somente às obrigações/regiões aprovadas. Captura incompleta não aprova referência; mudanças de configuração exigem referência correspondente. Seleção atual histórica permanece pendente, sem restauração automática para tornar o teste elegível.
- Análise estática Apps Script é determinística e limitada. Referências dinâmicas/ambíguas podem resultar Indeterminado. Fonte não é executada nem armazenada integralmente no histórico.
- Tokens ficam no cofre, mas SQLite contém metadados/cabeçalhos/fórmulas protegidas e hashes. Hash não é criptografia; proteção local depende do perfil/disco. Outro processo com controle do mesmo usuário é risco fora da promessa do cofre.
- Submissão de toast não comprova exibição/leitura. Crash entre submissão e confirmação SQLite pode repetir visualmente. Identidade instalada e clique físico exigem homologação própria; não repetir alerta sem autorização.
- Instalação em outra pasta compartilha o perfil/cofre. Prova local de instalar/desinstalar não comprova execução em Windows sem ferramentas de desenvolvimento.
- Autostart no login, assinatura/reputação, ambiente limpo e alertas hospedados GitHub não foram presumidos aprovados. O produto não exige servidor próprio. Publicação/distribuição a terceiros não está autorizada.

Consulte [relatório da Sprint05](sprint-05-report.md), [auditoria inicial](evidence/sprint06/security-report.md) e evidências novas em `docs/evidence/sprint06/`.


## Atualização vigente — integração e pedido de release, 03/11/2026

O usuário autorizou integrar `codex/sprint-06-homologation` e publicar um release instalável com OAuth Google verificado. Esta autorização substitui as restrições históricas de publicação acima, sem aprovar automaticamente a homologação nem comprovar verificação Google. O remoto `ba3e33d` foi mesclado preservando o histórico local `2664ed9` e o trabalho anteriormente não commitado; cópias integrais permanecem em stash local.

O candidato usa versão 1.0.0 conforme o escopo original; a pré-release histórica 0.1.0 permanece intacta. Não há configuração OAuth no ambiente Process/User/Machine nesta execução, e não foi fornecida evidência da verificação do projeto Google. Build sem cliente incorporado não deve ser publicado como release funcional. Carga Google real, Windows limpo e homologação humana permanecem pendentes. Evidências novas em `docs/evidence/release-integration/`.


### Atualização — audiência OAuth interna

O usuário informou que o console Google dispensa verificação porque o projeto está configurado com tipo de usuário Interno. Nesse cenário, não exigir aprovação pública como pré-condição: o OAuth limita-se aos membros da organização Google Cloud/Workspace do projeto e às políticas administrativas correspondentes. Fonte: https://support.google.com/cloud/answer/15549945#user-type-internal . A informação é relato do usuário; o nome/ID do projeto e o JSON Desktop ainda não foram fornecidos. O bloqueio atual é obter a configuração Desktop autorizada e comprovar login do candidato com conta interna elegível, além das homologações já pendentes. Não alterar a audiência para Externo nem ampliar escopos automaticamente. A futura release deve informar que o login é restrito à organização.
