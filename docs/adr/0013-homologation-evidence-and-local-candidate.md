# ADR 0013 — Evidência de homologação e candidato local V1.0

Estado: aceito para a Sprint06,03/11/2026. Fonte: docs/sprint06.original.md. Não substitui decisões nem evidências das sprints anteriores.

## Contexto

O código ativo tem provas locais e integração Google histórica, mas o candidato final ainda precisa de campanha integrada, carga e instalação. O usuário confirmou que não dispõe de Windows separado e30 recursos com referências aprovadas. A pasta de instalação não altera namespace do cofre/app_data_dir.

## Decisão

Separar evidência nova sintética, Windows real e serviços Google reais. Acrescentar campanha5/15/30 somente em testes Rust, sem IPC diagnóstico ou fixtures operacionais. Usar relógio injetado e tempo de parede identificados. Sink simulado prova submissão local, não exibição. Meta24h só conta com24h reais observadas.

Preparar versão1.0.0 como candidato local não publicado; versão não representa homologação. NSIS currentUser/x64, assets incorporados, avisos de terceiros e hashes. Sem cliente OAuth provisionado pelo mantenedor ou assinatura, registrar condições do artefato sem buscar credenciais no cofre. Não lançar produto instalado no perfil real. Prova limitada de instalação exige ausência de colisões em registros, atalhos e processos e remoção somente pelo uninstaller recém-gerado; execução limpa depende de ambiente separado.

O verificador passa a reconhecer o hash do Prompt06, exigir aceite explícito para conclusão e salvar resultados novos fora das pastas históricas. Não alterar critérios de aceite para adaptar ausência de ambiente. Enquanto requisitos essenciais não tiverem prova, classificar BLOQUEADA.

## Consequências

A sprint pode entregar testes, métricas e instalador sem aprovar distribuição. Google permanece somente leitura; baselines reais exigem humano; notificações e autostart não são acionados por testes locais. A homologação final exige recursos/consentimento/ambiente apropriados, mantendo pendências verificáveis.
