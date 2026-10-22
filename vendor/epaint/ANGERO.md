# epaint 0.36.2 — cópia com alterações do Office Angerô (Excel, Word, PowerPoint)

Cópia do crate [`epaint`](https://github.com/emilk/egui/tree/main/crates/epaint) 0.36.2
(MIT OR Apache-2.0, veja `LICENSE-MIT`), usada pelo `[patch.crates-io]` do `Cargo.toml` da raiz.

O que mudou (só no Windows):

- `src/text/dwrite.rs` (novo): cada letra é desenhada pelo DirectWrite, o motor de texto do Windows,
  com o encaixe GDI natural em ClearType (média das 3 cores = cinza) e um pouco mais de contraste,
  em vez do rasterizador do egui (letras regulares e bem espaçadas, como no Office). Se o DirectWrite não abrir a fonte, vale o desenho original.
- `src/text/font.rs`: `FontFace` guarda o índice da face; `allocate_glyph` tenta o DirectWrite
  primeiro; `hinted_advance_px` devolve a largura da letra ajustada aos pixels inteiros do GDI.
- `src/text/text_layout.rs`: o posicionamento usa essa largura ajustada (senão as letras encaixadas
  ficam com buracos entre si: "ST ELLA").
- `src/text/foco.rs` (novo, só no Excel): o "Foco do texto" (guia Exibir). 0 = automático (nível 3
  em telas de 125 % ou mais, nível 5 em telas de 100 %); 1 a 5 = manual, de suave a nítido. Mudar o
  nível faz o `Fonts` redesenhar todas as letras no quadro seguinte (`fonts.rs`, `geracao_foco`).
- `Cargo.toml`: dependência `windows` (DirectWrite) só no Windows.

`ANGERO_TEXTO=egui` volta ao desenho original (para comparar); `gdi` (o modo antigo: GDI clássico em
cinza), `ctgdi`, `natural` e `simetrico` usam outros modos do DirectWrite. `ANGERO_CONTRASTE` (padrão
0.5; 0 = sem reforço) escurece o traço.
