use godot::classes::TileMapLayer;
use godot::prelude::*;

use crate::domain::disparo::ResultadoDisparo;
use crate::domain::tabuleiro::{Celula, EstadoTabuleiro, BOARD_SIZE};
use crate::presentation::batalha::renderizacao_tabuleiro::atlas_tiles::{
    ATLAS_AGUA_1, ATLAS_AGUA_2, ATLAS_AGUA_ATINGIDA, 
    ATLAS_NAVIO_ATINGIDO, ATLAS_NAVIO_AFUNDADO,
};
use crate::presentation::batalha::renderizacao_tabuleiro::estilo_preview::{
    cor_preview_invalido, cor_preview_valido,
};
use crate::presentation::batalha::renderizacao_tabuleiro::navio_tiles::atlas_navio_por_nome;

// Retorna a sprite de água correta baseado no padrão xadrez
fn obter_sprite_agua(x: usize, y: usize) -> (i32, i32) {
    if (x + y) % 2 == 0 {
        ATLAS_AGUA_1
    } else {
        ATLAS_AGUA_2
    }
}

pub fn render_resultado_disparo(
    map: &mut Gd<TileMapLayer>,
    map_coord: Vector2i,
    resultado: &ResultadoDisparo,
) {
    match resultado {
        ResultadoDisparo::Agua => {
            map.set_cell_ex(map_coord)
                .source_id(0)
                .atlas_coords(Vector2i::new(ATLAS_AGUA_ATINGIDA.0, ATLAS_AGUA_ATINGIDA.1))
                .done();
        }
        ResultadoDisparo::Acerto => {
            map.set_cell_ex(map_coord)
                .source_id(0)
                .atlas_coords(Vector2i::new(ATLAS_NAVIO_ATINGIDO.0, ATLAS_NAVIO_ATINGIDO.1))
                .done();
        }
        ResultadoDisparo::Afundou(_) => {
            // Apenas renderiza a célula clicada
            // render_navio_afundado() deve ser chamado depois para renderizar o navio inteiro
            map.set_cell_ex(map_coord)
                .source_id(0)
                .atlas_coords(Vector2i::new(ATLAS_NAVIO_AFUNDADO.0, ATLAS_NAVIO_AFUNDADO.1))
                .done();
        }
        ResultadoDisparo::JaDisparado | ResultadoDisparo::ForaDosLimites => {}
    }
}

/// Renderiza todas as células de um navio afundado
pub fn render_navio_afundado(
    map: &mut Gd<TileMapLayer>,
    tabuleiro: &EstadoTabuleiro,
    navio_idx: usize,
) {
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if let Some(Celula::Afundado(idx)) = tabuleiro.valor_celula(x, y) {
                if idx == navio_idx {
                    let map_coord = Vector2i::new(y as i32, x as i32);
                    map.set_cell_ex(map_coord)
                        .source_id(0)
                        .atlas_coords(Vector2i::new(ATLAS_NAVIO_AFUNDADO.0, ATLAS_NAVIO_AFUNDADO.1))
                        .done();
                }
            }
        }
    }
}

pub fn render_tabuleiro_jogador(map: &mut Gd<TileMapLayer>, tabuleiro: &EstadoTabuleiro) {
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let map_coord = Vector2i::new(y as i32, x as i32);
            if let Some(celula) = tabuleiro.valor_celula(x, y) {
                match celula {
                    Celula::Ocupado(navio_idx) => {
                        let atlas_navio = tabuleiro
                            .navios
                            .get(navio_idx)
                            .map(|navio| atlas_navio_por_nome(&navio.nome))
                            .unwrap_or(atlas_navio_por_nome(""));
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(atlas_navio.0, atlas_navio.1))
                            .done();
                    }
                    Celula::AguaAtirada => {
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(ATLAS_AGUA_ATINGIDA.0, ATLAS_AGUA_ATINGIDA.1))
                            .done();
                    }
                    Celula::Atingido(_) => {
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(ATLAS_NAVIO_ATINGIDO.0, ATLAS_NAVIO_ATINGIDO.1))
                            .done();
                    }
                    Celula::Afundado(_) => {
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(ATLAS_NAVIO_AFUNDADO.0, ATLAS_NAVIO_AFUNDADO.1))
                            .done();
                    }
                    Celula::Vazio => {
                        // Renderizar água no padrão xadrez quando célula fica vazia (ex: navio removido)
                        let sprite_agua = obter_sprite_agua(x, y);
                        map.set_cell_ex(map_coord)
                            .source_id(0)
                            .atlas_coords(Vector2i::new(sprite_agua.0, sprite_agua.1))
                            .done();
                    }
                }
            }
        }
    }
}

pub fn render_preview_posicionamento(
    preview_map: &mut Gd<TileMapLayer>,
    nome_navio: &str,
    celulas: &[(usize, usize)],
    valido: bool,
) {
    preview_map.clear();

    if valido {
        preview_map.set_modulate(cor_preview_valido());
    } else {
        preview_map.set_modulate(cor_preview_invalido());
    }

    let atlas_navio = atlas_navio_por_nome(nome_navio);

    for (x, y) in celulas.iter() {
        preview_map
            .set_cell_ex(Vector2i::new(*y as i32, *x as i32))
            .source_id(0)
            .atlas_coords(Vector2i::new(atlas_navio.0, atlas_navio.1))
            .done();
    }
}

pub fn limpar_preview(preview_map: &mut Gd<TileMapLayer>) {
    preview_map.clear();
    preview_map.set_modulate(Color::WHITE);
}
