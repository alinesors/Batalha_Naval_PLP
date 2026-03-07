use godot::classes::TileMapLayer;
use godot::prelude::*;

use crate::domain::disparo::ResultadoDisparo;
use crate::domain::tabuleiro::{Celula, EstadoTabuleiro, BOARD_SIZE};
use crate::presentation::tabuleiro_visual::atlas_tiles::{ATLAS_ACERTO, ATLAS_AGUA, ATLAS_NAVIO};
use crate::presentation::tabuleiro_visual::estilo_preview::{
    cor_preview_invalido, cor_preview_valido,
};

pub fn render_resultado_disparo(
    map: &mut Gd<TileMapLayer>,
    map_coord: Vector2i,
    resultado: &ResultadoDisparo,
) {
    match resultado {
        ResultadoDisparo::Agua => {
            map.set_cell_ex(map_coord)
                .source_id(0)
                .atlas_coords(Vector2i::new(ATLAS_AGUA.0, ATLAS_AGUA.1))
                .done();
        }
        ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_) => {
            map.set_cell_ex(map_coord)
                .source_id(0)
                .atlas_coords(Vector2i::new(ATLAS_ACERTO.0, ATLAS_ACERTO.1))
                .done();
        }
        ResultadoDisparo::JaDisparado | ResultadoDisparo::ForaDosLimites => {}
    }
}

pub fn render_tabuleiro_jogador(map: &mut Gd<TileMapLayer>, tabuleiro: &EstadoTabuleiro) {
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let map_coord = Vector2i::new(x as i32, y as i32);
            if let Some(celula) = tabuleiro.valor_celula(x, y) {
                match celula {
                    Celula::Ocupado(_) => {
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(ATLAS_NAVIO.0, ATLAS_NAVIO.1))
                            .done();
                    }
                    Celula::Agua => {
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(ATLAS_AGUA.0, ATLAS_AGUA.1))
                            .done();
                    }
                    Celula::Atingido(_) => {
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(ATLAS_ACERTO.0, ATLAS_ACERTO.1))
                            .done();
                    }
                    Celula::Vazio => {}
                }
            }
        }
    }
}

pub fn render_preview_posicionamento(
    preview_map: &mut Gd<TileMapLayer>,
    celulas: &[(usize, usize)],
    valido: bool,
) {
    preview_map.clear();

    if valido {
        preview_map.set_modulate(cor_preview_valido());
    } else {
        preview_map.set_modulate(cor_preview_invalido());
    }

    for (x, y) in celulas.iter() {
        preview_map
            .set_cell_ex(Vector2i::new(*x as i32, *y as i32))
            .source_id(0)
            .atlas_coords(Vector2i::new(ATLAS_NAVIO.0, ATLAS_NAVIO.1))
            .done();
    }
}

pub fn limpar_preview(preview_map: &mut Gd<TileMapLayer>) {
    preview_map.clear();
    preview_map.set_modulate(Color::WHITE);
}
