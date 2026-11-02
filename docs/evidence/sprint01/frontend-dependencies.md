# Dependências frontend — Sprint 01

A resolução usa `package-lock.json` e npm; não houve migração para pnpm. O código próprio usa MIT. As licenças das bibliotecas permanecem aplicáveis a cada dependência.

ESLint 10, typescript-eslint e @eslint/config-array ainda resolviam minimatch 10.2.6. O advisory oficial [GHSA-g3hv-75wg-m249](https://github.com/advisories/GHSA-g3hv-75wg-m249), consultado nesta execução, lista todas as versões <11.0.1 como afetadas e 11.0.1 como corrigida. Não há versão corrigida dentro do major 10 conforme esse advisory.

Foi aplicado override mínimo de minimatch ^11.0.1 somente aos três consumidores de desenvolvimento: eslint, @eslint/config-array e @typescript-eslint/typescript-estree. Não foi usado `npm audit fix --force` nem o downgrade para ESLint 4 sugerido pelo npm audit. O override pode ser removido quando os consumidores adotarem a versão corrigida. A compatibilidade relevante é verificada com instalação pelo lock, lint real de TS/TSX/Vite, typecheck, testes e build. Isso não constitui prova de equivalência de toda a API de glob.

Os logs `frontend-audit-initial.json`, `frontend-tooling-upgrade.log`, `frontend-minimatch-upgrade.log` e os checks finais preservam a evidência. A saída final de audit deve ser interpretada no instante da execução, sem promessa sobre vulnerabilidades futuras.
