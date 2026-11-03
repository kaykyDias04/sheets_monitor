# Plano de empacotamento — Sprint 06

Inspeção em 03/11/2026. Estado: plano anterior à implementação; nenhum instalador foi gerado ou executado por esta lane. Escopo: SM-028, docs/sprint06.original.md; limites de isolamento do Gate 1 preservados.

## Candidato local

Preparar versão 1.0.0 nos manifestos npm/Rust/Tauri, sem tag, release pública ou distribuição a terceiros. Manter identifier com.sheetsmonitor.desktop e as migrações 0001–0005. A versão do candidato não significa homologação da V1.0. Configurar bundle NSIS ativo, Windows x64/MSVC, instalação currentUser, allowDowngrades=false e idiomas PortugueseBR/English. Manter CSP/capabilities e native-probe fora do produto padrão.

O instalador de produção pode usar o mecanismo oficial downloadBootstrapper para WebView2. A prova local só deve ser iniciada se o runtime já estiver disponível e a revisão do script NSIS gerado confirmar o caminho sem download/elevação. Não instalar outro runtime ou SDK silenciosamente para esconder requisito ausente.

O build incorpora assets locais e precisa funcionar sem Vite. O helper deve importar o ambiente MSVC revisado, usar Cargo.lock/package-lock existentes, executar Tauri release para x86_64-pc-windows-msvc com --locked e manter stdout/stderr em evidência nova. Não executar o produto automaticamente ao final do build.

## Avisos de terceiros

Gerar inventário a partir de cargo metadata --locked --filter-platform x86_64-pc-windows-msvc com features custom-protocol e da resolução npm instalada/lockfile. Percorrer a closure Rust normal/build, excluindo dev, e os pacotes npm de produção efetivamente disponíveis. Incluir fontes de avisos e textos LICENSE/COPYING/NOTICE/COPYRIGHT encontrados nos pacotes, com hashes; preservar licenças específicas como Apache, Unicode, MPL e atribuições, sem reinterpretá-las como MIT.

Pode ser utilizado um conjunto conservador maior que o código efetivamente ligado, desde que o inventário declare esse limite; a análise de árvore não comprova automaticamente quais bytes foram distribuídos. Ausência de texto/licença ou pacote não resolvido precisa gerar lista explícita de pendências. SPDX sozinho não substitui aviso obrigatório. Não afirmar auditoria jurídica completa.

Incluir LICENSE próprio e THIRD-PARTY-NOTICES no bundle por resources map. Não redistribuir credenciais, toolchains ou SDKs. O helper registra nomes/versões/licenças e caminhos relativos públicos, sem caminhos privados de cache do perfil.

## Provisão OAuth e assinatura

Somente variáveis de ambiente de build expressamente disponibilizadas podem provisionar o cliente Desktop. Não ler cofre, arquivos OAuth existentes nem copiar client config do usuário. Registrar apenas oauth_provisioned=true/false e desktop_secret_present=true/false, nunca seus valores. Sem provisão, o artefato é um candidato local não pronto para primeiro acesso de novos usuários, mesmo se uma sessão local antiga funcionar.

Sem certificado/serviço de assinatura autorizado disponível, gerar candidato local unsigned e registrar esse estado. Não contornar SmartScreen ou solicitar certificado/senha no chat. Assinatura/reputação e distribuição externa permanecem checkpoint separado.

## Prova de instalação/desinstalação limitada

Antes de qualquer execução do instalador, verificar:

1. HKCU/HKLM, nas visões 32/64 bits, para chaves de uninstall referentes ao identifier, produto, executável ou diretório padrão; recusar qualquer colisão.
2. Atalhos existentes com identidade/nome do produto em desktop e Start Menu de usuário e máquina; preservar e recusar colisões.
3. Processos do produto já em execução; não encerrar processos do usuário. Se houver instância, registrar bloqueio da prova.
4. Diretório absoluto isolado e novo, restrito a .tooling/sprint06-install/<identificador>. Recusar diretório existente/não vazio; não efetuar limpeza recursiva genérica.
5. Runtime WebView2 previamente disponível e script NSIS gerado inspecionado; nenhum lançamento pós-instalação em modo silencioso.

Executar somente o instalador candidato cujo SHA-256 já foi calculado, usando /S e /D=<diretório absoluto isolado> como último argumento, conforme NSIS. Registrar exit code, presença/hash/versão/arquitetura do executável, avisos empacotados, registro de uninstall e atalhos/AppUserModelID quando acessíveis. **Não lançar o executável instalado:** o app_data_dir, cofre e instância única continuam usando identidade de produção independentemente do diretório de instalação.

Desinstalar somente pelo uninstaller criado pela prova, validado dentro do diretório isolado. Registrar saída e remoção dos arquivos/atalhos/registro novos. Não apagar app_data_dir, credenciais ou arquivos Google; nenhuma aprovação de baseline, toast ou autostart. Verificar que a prova não criou entrada Run nova. A prova limitada confirma mecânica de instalação/desinstalação no Windows atual, não execução em Windows limpo nem login/notificação instalada.

O usuário confirmou indisponibilidade de VM/outro Windows e 30 referências aprovadas. Instalação executada, quando segura, deve permanecer separada desses critérios não comprovados. A sprint continua BLOQUEADA enquanto requisitos essenciais estiverem sem prova.

## Artefatos e evidência

Salvar instalador em src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis (ignorado no Git), hash SHA-256/bytes/versão e estado unsigned em docs/evidence/sprint06/distribution-build.json. Salvar logs e installation-validation.json com resultados reais ou razões de bloqueio. Relatório final deve apontar artefato exato e instruções reprodutíveis, sem publicar ou distribuir.

## Fontes oficiais consultadas

- [Tauri — Windows Installer](https://v2.tauri.app/distribute/windows-installer/): NSIS, currentUser e opções WebView2.
- [Tauri — Configuration](https://v2.tauri.app/reference/config/): bundle.resources, licenseFile, NSIS installMode e Windows allowDowngrades. O schema local node_modules/@tauri-apps/cli/config.schema.json confirma os campos da versão instalada.
- [NSIS — Command Line Usage](https://nsis.sourceforge.io/Docs/Chapter3.html): /S sensível a maiúsculas, /D como último argumento e caminho de instalação.
- [NSIS — Scripting Reference](https://nsis.sourceforge.io/Docs/Chapter4.html): comportamento de instalação/desinstalação silenciosas. O template efetivo gerado pelo Tauri deve ser inspecionado antes de executar.
