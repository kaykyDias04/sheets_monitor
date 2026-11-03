# Adaptadores nativos — testes locais

2026-11-02, ambiente Windows MSVC. Nenhuma notificação real, entrada autostart ou configuração Google foi modificada por estes comandos.

- `cargo check --manifest-path src-tauri/Cargo.toml --lib`: aprovado, 1m51s, novas dependências desktop compiladas. A execução inicial apontou somente unused_mut em app.rs:295, informado à lane proprietária para correção.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib desktop:: -- --nocapture`: **4 aprovados, 0 falhas, 236 filtrados**, execução 0.00s após compilação 3m58s. Aguardou lock do target compartilhado antes de compilar.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib notifications::native:: -- --nocapture`: **1 aprovado, 0 falhas, 239 filtrados**, execução 0.00s, recompilação 6.42s. Payload contém aba conhecida, severidade, nome/hora; remove controles, limita comprimento e não inclui ID Google bruto no texto.

Cobertura do adaptador autostart usa armazenamento simulado: observação não grava; opt-in/remoção verificam estado observado; entrada de outra instalação é preservada; caminho é citado e limites/rejeições aplicados; falha de leitura não autoriza mutação; escrita ignorada não é reportada como sucesso. Esses testes não habilitam autostart real nem comprovam execução no login.

Pesquisa/limites oficiais em docs/sprint05-native-research.md. Teste do payload e inspeção Windows são registrados separadamente; testes simulados não comprovam exibição de toast, bandeja ou clique real.
