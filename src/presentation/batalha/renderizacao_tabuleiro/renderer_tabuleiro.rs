use godot::classes::TileMapLayer;
use godot::prelude::*;

use crate::domain::disparo::ResultadoDisparo;
use crate::domain::tabuleiro::{Celula, EstadoTabuleiro, BOARD_SIZE};
use crate::presentation::batalha::renderizacao_tabuleiro::atlas_tiles::{
    ATLAS_AGUA_1, ATLAS_AGUA_2, ATLAS_AGUA_ATINGIDA, ATLAS_HIT_MARKER,
    COL_INTACTO, COL_DANIFICADO, COL_DESTRUIDO,
    SOURCE_AGUA,
};
use crate::presentation::batalha::renderizacao_tabuleiro::estilo_preview::{
    cor_preview_invalido, cor_preview_valido,
};
use crate::presentation::batalha::renderizacao_tabuleiro::navio_tiles::atlas_segmento_navio;

fn obter_sprite_agua(x: usize, y: usize) -> (i32, i32) {
    if (x + y) % 2 == 0 { ATLAS_AGUA_1 } else { ATLAS_AGUA_2 }
}

fn obter_info_segmento(
    tabuleiro: &EstadoTabuleiro,
    navio_idx: usize,
    cx: usize,
    cy: usize,
) -> Option<(usize, bool, usize)> {
    let tamanho = tabuleiro.navios.get(navio_idx)?.tamanho;
    let mut celulas = tabuleiro.obter_celulas_navio(navio_idx);
    if celulas.is_empty() {
        return None;
    }

    let first_x = celulas[0].0;
    let horizontal = celulas.iter().all(|&(x, _)| x == first_x);

    if horizontal {
        celulas.sort_by_key(|&(_, y)| y);
    } else {
        celulas.sort_by_key(|&(x, _)| x);
    }

    let segmento = celulas.iter().position(|&(x, y)| x == cx && y == cy)?;
    Some((segmento, horizontal, tamanho))
}

// `board_map` recebe apenas água e marcadores de tiro (nunca é apagado por navios).
// `ship_map`  recebe os sprites dos navios por segmento/estado/orientação.
pub fn render_tabuleiro_jogador(
    board_map: &mut Gd<TileMapLayer>,
    ship_map: &mut Gd<TileMapLayer>,
    tabuleiro: &EstadoTabuleiro,
) {
    ship_map.clear();

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let map_coord = Vector2i::new(y as i32, x as i32);
            let Some(celula) = tabuleiro.valor_celula(x, y) else { continue };

            match celula {
                Celula::Ocupado(navio_idx) => {
                    if let Some((seg, horiz, tam)) =
                        obter_info_segmento(tabuleiro, navio_idx, x, y)
                    {
                        let (source, atlas, alt) =
                            atlas_segmento_navio(tam, seg, COL_INTACTO, horiz);
                        ship_map
                            .set_cell_ex(map_coord)
                            .source_id(source)
                            .atlas_coords(atlas)
                            .alternative_tile(alt)
                            .done();
                    }
                }
                Celula::Atingido(navio_idx) => {
                    if let Some((seg, horiz, tam)) =
                        obter_info_segmento(tabuleiro, navio_idx, x, y)
                    {
                        let (source, atlas, alt) =
                            atlas_segmento_navio(tam, seg, COL_DANIFICADO, horiz);
                        ship_map
                            .set_cell_ex(map_coord)
                            .source_id(source)
                            .atlas_coords(atlas)
                            .alternative_tile(alt)
                            .done();
                    }
                    board_map
                        .set_cell_ex(map_coord)
                        .source_id(SOURCE_AGUA)
                        .atlas_coords(Vector2i::new(ATLAS_HIT_MARKER.0, ATLAS_HIT_MARKER.1))
                        .done();
                }
                Celula::Afundado(navio_idx) => {
                    if let Some((seg, horiz, tam)) =
                        obter_info_segmento(tabuleiro, navio_idx, x, y)
                    {
                        let (source, atlas, alt) =
                            atlas_segmento_navio(tam, seg, COL_DESTRUIDO, horiz);
                        ship_map
                            .set_cell_ex(map_coord)
                            .source_id(source)
                            .atlas_coords(atlas)
                            .alternative_tile(alt)
                            .done();
                    }
                }

                Celula::AguaAtirada => {
                    board_map
                        .set_cell_ex(map_coord)
                        .source_id(SOURCE_AGUA)
                        .atlas_coords(Vector2i::new(ATLAS_AGUA_ATINGIDA.0, ATLAS_AGUA_ATINGIDA.1))
                        .done();
                }

                Celula::Vazio => {
                    let sprite = obter_sprite_agua(x, y);
                    board_map
                        .set_cell_ex(map_coord)
                        .source_id(SOURCE_AGUA)
                        .atlas_coords(Vector2i::new(sprite.0, sprite.1))
                        .done();
                }
            }
        }
    }
}

/// Ao afundar um navio no tabuleiro inimigo, mostra os sprites no ship_map.
pub fn render_navio_afundado(
    ship_map: &mut Gd<TileMapLayer>,
    tabuleiro: &EstadoTabuleiro,
    navio_idx: usize,
) {
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if let Some(Celula::Afundado(idx)) = tabuleiro.valor_celula(x, y) {
                if idx == navio_idx {
                    if let Some((seg, horiz, tam)) =
                        obter_info_segmento(tabuleiro, navio_idx, x, y)
                    {
                        let map_coord = Vector2i::new(y as i32, x as i32);
                        let (source, atlas, alt) =
                            atlas_segmento_navio(tam, seg, COL_DESTRUIDO, horiz);
                        ship_map
                            .set_cell_ex(map_coord)
                            .source_id(source)
                            .atlas_coords(atlas)
                            .alternative_tile(alt)
                            .done();
                    }
                }
            }
        }
    }
}

pub fn render_resultado_disparo(
    board_map: &mut Gd<TileMapLayer>,
    map_coord: Vector2i,
    resultado: &ResultadoDisparo,
) {
    match resultado {
        ResultadoDisparo::Agua => {
            board_map
                .set_cell_ex(map_coord)
                .source_id(SOURCE_AGUA)
                .atlas_coords(Vector2i::new(ATLAS_AGUA_ATINGIDA.0, ATLAS_AGUA_ATINGIDA.1))
                .done();
        }
        ResultadoDisparo::Acerto => {
            board_map
                .set_cell_ex(map_coord)
                .source_id(SOURCE_AGUA)
                .atlas_coords(Vector2i::new(ATLAS_HIT_MARKER.0, ATLAS_HIT_MARKER.1))
                .done();
        }
        ResultadoDisparo::Afundou(_) => {
            board_map
                .set_cell_ex(map_coord)
                .source_id(SOURCE_AGUA)
                .atlas_coords(Vector2i::new(ATLAS_HIT_MARKER.0, ATLAS_HIT_MARKER.1))
                .done();
        }
        ResultadoDisparo::JaDisparado | ResultadoDisparo::ForaDosLimites => {}
    }
}

pub fn render_preview_posicionamento(
    preview_map: &mut Gd<TileMapLayer>,
    _nome_navio: &str,
    celulas: &[(usize, usize)],
    valido: bool,
) {
    preview_map.clear();
    preview_map.set_modulate(if valido { cor_preview_valido() } else { cor_preview_invalido() });

    let tamanho = celulas.len();
    if tamanho == 0 { return; }

    let first_x = celulas[0].0;
    let horizontal = celulas.iter().all(|&(x, _)| x == first_x);

    let mut ordenadas: Vec<(usize, usize)> = celulas.to_vec();
    if horizontal {
        ordenadas.sort_by_key(|&(_, y)| y);
    } else {
        ordenadas.sort_by_key(|&(x, _)| x);
    }

    for (segmento, &(x, y)) in ordenadas.iter().enumerate() {
        let (source, atlas, alt) =
            atlas_segmento_navio(tamanho, segmento, COL_INTACTO, horizontal);
        preview_map
            .set_cell_ex(Vector2i::new(y as i32, x as i32))
            .source_id(source)
            .atlas_coords(atlas)
            .alternative_tile(alt)
            .done();
    }
}

pub fn limpar_preview(preview_map: &mut Gd<TileMapLayer>) {
    preview_map.clear();
    preview_map.set_modulate(Color::WHITE);
}
