//! Angerô: the text-sharpness setting ("foco"), shared by the app and the Windows rasterizer.
//!
//! Level 0 is automatic: each screen gets the level that suits its pixel density (see
//! [`foco_efetivo`]). Levels 1–5 go from smooth (soft edges, like a web browser) to sharp (strong
//! grid-fitting, like classic Windows). Changing it bumps [`geracao`], which makes `Fonts` redraw
//! every glyph and lay the text out again on the next frame.

use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

/// The most manual level ([`definir_foco`] clamps to it).
pub const FOCO_MAXIMO: u8 = 5;

static NIVEL: AtomicU8 = AtomicU8::new(0);
static GERACAO: AtomicU32 = AtomicU32::new(0);

/// Sets the sharpness: 0 = automatic, 1 (smooth) ..= [`FOCO_MAXIMO`] (sharp).
pub fn definir_foco(nivel: u8) {
    let nivel = nivel.min(FOCO_MAXIMO);
    if NIVEL.swap(nivel, Ordering::Relaxed) != nivel {
        GERACAO.fetch_add(1, Ordering::Relaxed);
    }
}

/// The level set with [`definir_foco`] (0 = automatic).
pub fn foco() -> u8 {
    NIVEL.load(Ordering::Relaxed)
}

/// Changes whenever the setting does.
pub fn geracao() -> u32 {
    GERACAO.load(Ordering::Relaxed)
}

/// The level actually used on a screen with `pixels_per_point`: the manual one, or in automatic
/// mode 3 on high-density screens (125 % and up, where soft edges already look crisp) and 5 on
/// 100 % screens (where every edge pixel shows).
pub fn foco_efetivo(pixels_per_point: f32) -> u8 {
    match foco() {
        0 if pixels_per_point >= 1.2 => 3,
        0 => 5,
        n => n,
    }
}
