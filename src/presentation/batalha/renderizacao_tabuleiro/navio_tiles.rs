use godot::prelude::Vector2i;

use crate::presentation::batalha::renderizacao_tabuleiro::atlas_tiles::{
    ALT_HORIZONTAL, ALT_VERTICAL,
    NAVIO_1X1_BASE_ROW, NAVIO_1X3_BASE_ROW, NAVIO_1X4_BASE_ROW, NAVIO_1X6_BASE_ROW,
    SOURCE_NAVIOS,
};


pub fn base_row(tamanho: usize) -> i32 {
    match tamanho {
        1 => NAVIO_1X1_BASE_ROW,
        3 => NAVIO_1X3_BASE_ROW,
        4 => NAVIO_1X4_BASE_ROW,
        6 => NAVIO_1X6_BASE_ROW,
        _ => NAVIO_1X6_BASE_ROW,
    }
}

pub fn atlas_segmento_navio(
    tamanho: usize,
    segmento: usize,
    col_estado: i32,
    horizontal: bool,
) -> (i32, Vector2i, i32) {
    let row = base_row(tamanho) + segmento as i32;
    let atlas_coords = Vector2i::new(col_estado, row);
    let alternative = if horizontal { ALT_HORIZONTAL } else { ALT_VERTICAL };
    (SOURCE_NAVIOS, atlas_coords, alternative)
}
