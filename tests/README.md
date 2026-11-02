# Matriz de verificação da fundação

Os testes executáveis ficam junto aos módulos que verificam; este diretório organiza a verificação integrada/manual da Sprint 01.

| Camada | Comando / roteiro | Evidência |
|---|---|---|
| UI e bridge de demonstração | `npm test` — navegação, filtros, preferências, erros e foco de diálogo | `docs/evidence/sprint01/frontend-tests.log` |
| Tipos, lint e formato | scripts npm e `check-rust.ps1 -Task format-check` | logs frontend e native-gates |
| Banco unitário | módulo `src-tauri/src/database` | rust-test.log (subconjunto de testes do crate) |
| Persistência integrada | `src-tauri/tests/database_persistence.rs` | reabertura, ledger e versão futura recusada |
| Shell/DTO nativo | `check-rust.ps1 -Task test` | dashboard serializado com fixtures e sem campos de tokens |
| Repositório | `node scripts/verify-repository.mjs` | original/hash/IDs/links/arquivo/escopo/padrões de credenciais |
| Executável Windows | roteiro abaixo | native-qa.json, native-qa.md, capturas do renderer |

## Roteiro nativo

1. Compilar o frontend e o executável standalone com custom-protocol; conferir que Vite não é necessário.
2. Abrir `sheets-monitor.exe` em Windows 11 e conferir título de demonstração e janela real.
3. Navegar Dashboard, Planilhas, Histórico e Configurações. Conferir quatro estados fictícios e Google indisponível.
4. Salvar intervalo120 pela UI; conferir preferências no SQLite da fundação em modo readonly.
5. Fechar e reabrir a instância; conferir120 na UI/banco e os mesmos quatro arquivos, sem duplicação de seed.
6. Restaurar60 pela interface. Verificar que autostart não pode ser ativado e nenhuma credencial foi solicitada.
7. Inspecionar legibilidade e acessibilidade. Relatar limitações do método de captura; auditoria automatizada não certifica toda acessibilidade.

Se for usado CDP, habilitar porta loopback e perfil isolado apenas no ambiente do processo de QA. Fechar essa instância ao terminar e abrir normalmente, sem flags diagnósticas. Não alterar registro nem variáveis de usuário/máquina. Não usar nem salvar uma captura de outro aplicativo como evidência.

Esses casos não são testes de OAuth, Google real, integridade, scheduler, bandeja, notificações ou instalador. Cenários completos de produto pertencem a SM-026 após autorização das sprints correspondentes.
