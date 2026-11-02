# Verificação — Sprint 02

Executar no Windows 11 x64, na raiz do repositório:

```powershell
npm ci
npm run lint
npm run format:check
npm run typecheck
npm test
npm run build
.\scripts\check-rust.ps1 -Task format-check
.\scripts\check-rust.ps1 -Task test
.\scripts\check-rust.ps1 -Task clippy
.\scripts\check-rust.ps1 -Task build
node scripts/verify-repository.mjs
```

O helper Rust importa MSVC e escreve em `docs/evidence/sprint02/`; `-EvidenceDirectory` permite outro diretório. Não sobrescrever evidências da Sprint 01. Mocks e fixtures ficam em testes, sem comando IPC para habilitar dados falsos como resultado real.

A suíte Rust cobre PKCE/state/host/callback, consentimento negado, exchange/refresh com transporte simulado, invalidação e identidade, revogação explícita, segredo fora de DTO/Debug, parsers e respostas dos três serviços, hashes, erros/retries/timeouts/tamanho e persistência por conta. SQLite verifica upgrade da migração1 para2 e preservação das preferências; a migração1 permanece intacta. A suíte React verifica navegação da fundação, ações de conta, descarte de conta divergente, limpeza de segredo/resultados, erros fechados e confirmação de revogação.

Teste adicional do cofre Windows, com bytes sintéticos e namespace UUID exclusivo:

```powershell
. .\scripts\environment-common.ps1
Update-ProcessToolPath
Import-CppEnvironment (Get-CppInstallation)
cargo test --manifest-path src-tauri/Cargo.toml --locked --target x86_64-pc-windows-msvc --features custom-protocol windows_vault_roundtrip -- --ignored
```

Esse teste exercita armazenamento real do Windows, substituição e limpeza apenas das entradas de teste. Não comprova tokens Google reais. O teste é ignorado na suíte padrão para evitar alterar cofre em execução não explícita.

Prova nativa deve abrir o executável com assets incorporados, confirmar bridge Tauri, migração2, cinco telas e estado desconectado sem fixtures Google. Capturas desconectadas podem ser versionadas. Instrumentação temporária de WebView deve escutar apenas loopback e ser encerrada ao terminar. Não registrar authorization URL, code, state, tokens, identidade ou conteúdo real em logs públicos.

Homologação externa separada: seguir [oauth-setup](oauth-setup.md), consentir no navegador, ler abas/valores/fórmulas de uma planilha fictícia autorizada, metadados Drive com opt-in, projeto Apps Script e fontes. Reiniciar e consultar novamente para renovar a sessão recuperada. Registrar só checks booleanos/categorias sanitizadas. Sem acesso autorizado, marcar pendente, nunca aprovado por mocks.

CI Windows usa os mesmos gates, mas existência do workflow não prova execução hospedada. As falhas iniciais ficam preservadas; resultados atuais e limitações constam em [progress](progress.md).

Executar o build standalone por último antes da prova nativa: `cargo test` com features padrão pode substituir o executável pelo modo development, que aponta para localhost:1420. O comando adicional de cofre acima usa custom-protocol para preservar esse modo. Não concluir sucesso standalone se o renderer depender do Vite.
