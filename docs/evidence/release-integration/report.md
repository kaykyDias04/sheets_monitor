# Integração e candidato de release — 03/11/2026

## Escopo e proveniência

Pedido explícito: trazer `origin/codex/sprint-06-homologation`, conciliar alterações locais, integrar em main e publicar aplicativo instalável com OAuth Google verificado.

- Base main: `b2c68ce0791aa8acdcc860f433dd2ed6a61849da`.
- Head local inicial: `2664ed9`; remoto recebido: `ba3e33d270d346a51474b2afeb4962bfad38fad5`.
- Merge de ambas as histórias: `f5f5272c8ba04f400e4bf5c0e16b3cd434d91965`.
- Trabalho não commitado preservado integralmente em dois stashes locais, reconciliado sem reset/force-push. Inventário de licenças local anterior preservado em `local-premerge-distribution-licenses.json`; manual local preservado como `docs/installation-local-candidate.md`.
- Migrações0001–0005, protótipo e evidências Sprint01–05 sem alterações. Pré-release pública histórica `v0.1.0-sprint06-preview` preservada.

## Correções de integração

Conflitos de workflow, governança/backlog/progresso, verificadores e caminho de evidência reconciliados. Mantidos estados bloqueados de homologação real. Verificador preserva requisito de evidência humana e usa diretório novo. Versão1.0.0 do candidato local preservada nos manifests/lockfiles. Colisão de propriedades JSON no empacotamento corrigida: um mapa de recursos preserva LICENSE, avisos completos, avisos Rust e manual. ADR local renumerado0013 para preservar0012 remoto. Manual local corrigido para usar comando Tauri existente.

## Evidência local nova

- Frontend: lint, Prettier, TypeScript,50 testes/10 arquivos e build Vite aprovados. Logs `frontend-*.log`.
- Rust Windows x64 MSVC:261 unitários e2 integrações aprovados,1 teste de cofre ignorado por padrão. A suíte inclui três cenários integrados sintéticos, inclusive5/15/30. Logs em `rust/`; não é carga Google nem24h.
- Rustfmt e Clippy com warnings negados aprovados.
- Verificador do repositório aprovado; links e padrões conhecidos de credenciais revisados. Este scanner limitado não comprova ausência universal de segredos.
- npm audit produção:0 vulnerabilidades conhecidas reportadas em `npm-audit-production.json`.
- GitHub secret scanning:0 alertas abertos. Dependabot e code scanning indisponíveis à consulta; não declarar estado limpo desses serviços (`github-security-alerts.json`). A revisão histórica OSV do grafo Rust é reutilizada somente como contexto, sem afirmar nova consulta de advisories.
- Revisão independente: gates1/3 aprovaram fonte/integração; gate2 confirmou correção do mapa Tauri e bloqueio OAuth. Revisão não é certificação de segurança.

## Limites de publicação

O usuário autorizou publicação; não é necessária nova autorização para publicar quando os critérios estiverem comprovados. Entretanto, não há cliente OAuth disponível em Process/User/Machine, nem JSON Desktop em Downloads, e o projeto Google autorizado/verificação ainda não foi identificado nesta execução. O cofre não foi lido para fabricar provisionamento. Usuário recebeu instruções para consultar Central de verificação e localizar cliente Desktop; não deve colar conteúdo privado no chat.

Login histórico no perfil do mantenedor não prova primeiro acesso do novo pacote. Build sem cliente não pode ser anunciado como release funcional. Verificação/aprovação Google, primeiro login do candidato, carga Google real, ambiente Windows limpo e homologação humana continuam pendentes. Não houve aprovação de baseline, acesso Google novo, toast, autostart, instalação ou abertura do candidato no perfil real. A nova compilação NSIS será descrita no manifesto de artefatos; sua geração não comprova os critérios pendentes.

Fontes Google consultadas: https://developers.google.com/identity/protocols/oauth2/native-app e https://developers.google.com/identity/protocols/oauth2/production-readiness/overview . Em produção e verificado são estados distintos.

Estado: integração validada localmente; release funcional com OAuth verificado BLOQUEADA por configuração/evidências externas indisponíveis. Nenhum novo release/tag publicado nesta execução.

## Compilação do candidato conciliado

Build Tauri release MSVC x64 concluído, NSIS gerado com exit0. Manifesto e SHA256SUMS.txt no mesmo diretório deste relatório. Pacote não assinado; OAuth não provisionado; não foi instalado/aberto nesta execução. Inspeção estática PE/imports registrada, sem alegar execução em Windows limpo. Nenhuma nova release/tag publicada. CI do merge em main deve ser consultado no GitHub pelo SHA final; resultados de branches anteriores não comprovam esse SHA.


### Atualização — audiência OAuth interna

O usuário informou que o console Google dispensa verificação porque o projeto está configurado com tipo de usuário Interno. Nesse cenário, não exigir aprovação pública como pré-condição: o OAuth limita-se aos membros da organização Google Cloud/Workspace do projeto e às políticas administrativas correspondentes. Fonte: https://support.google.com/cloud/answer/15549945#user-type-internal . A informação é relato do usuário; o nome/ID do projeto e o JSON Desktop ainda não foram fornecidos. O bloqueio atual é obter a configuração Desktop autorizada e comprovar login do candidato com conta interna elegível, além das homologações já pendentes. Não alterar a audiência para Externo nem ampliar escopos automaticamente. A futura release deve informar que o login é restrito à organização.
