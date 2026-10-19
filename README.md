# Excel - Angerô

Programa do **Office Angerô**, o pacote de escritório do **Grupo Angerô**.

Planilhas: fórmulas, gráficos, tabelas dinâmicas e filtros; abre e salva .xlsx.

- Ajuda: https://github.com/wesleiad/Office-Angero/blob/main/docs/ajuda/excel.md
- Office Angerô (ícones, instaladores, ajuda): https://github.com/wesleiad/Office-Angero
- Versões compiladas: aba **Releases** deste repositório (o programa instalado se atualiza sozinho a partir delas).

## Compilar

Windows, com Rust (MSVC) e Visual Studio Build Tools (C++):

```
cargo build --release --locked -p gridcraft -p gridcraft-cli
```

Gera `target/release/excel-angero.exe` e `target/release/excel-angero-cli.exe`.
A cada push no ramo `angero`, o GitHub Actions compila e publica uma versão nova em Releases.

## Código-base e licença

Desenvolvido por Weslei Domingues (TI) para o Grupo Angerô, a partir do código aberto do
**GridCraft** (projeto ArtCraft, https://github.com/storytold/gridcraft), usado como base.
Licença: MIT OU Apache-2.0, à escolha — veja `LICENSE-MIT`, `LICENSE-APACHE` e `NOTICE`.
