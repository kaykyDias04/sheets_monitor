# ADR 0002 — toolchain portátil de verificação

Status: aceito em 01/11/2026.

O ambiente tinha Node.js, mas não Rust/Cargo nem MSVC C++ Build Tools. A presença de Windows Kits era somente de ferramentas de diagnóstico, sem SDK de compilação.

Rustup foi instalado no perfil do usuário sem alterar PATH global e sem elevação. Para verificar o núcleo sem solicitar administrador, baixamos LLVM-MinGW portátil do repositório oficial mstorsjo/llvm-mingw para `.tooling/`, ignorado pelo Git. Os comandos de verificação GNU usam variáveis de ambiente restritas ao processo.

O caminho de distribuição documentado e de CI continua Windows MSVC com os pré-requisitos oficiais do Tauri. Resultados GNU não serão apresentados como validação do instalador MSVC. Toolchains/dependências não são copiadas para o repositório.

Referências: https://v2.tauri.app/start/prerequisites/ e https://github.com/mstorsjo/llvm-mingw/releases.
