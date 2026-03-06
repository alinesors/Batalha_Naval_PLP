use godot::classes::{INode2D, InputEvent, InputEventMouseButton, Node2D, TileMapLayer};
use godot::global::MouseButton;
use godot::prelude::*;

use crate::domain::disparo::{ResultadoDisparo, executar_disparo};
use crate::domain::tabuleiro::{BOARD_SIZE, EstadoTabuleiro, Celula};

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct ControladorBatalha {
    tabuleiro_jogador: EstadoTabuleiro,
    tabuleiro_inimigo: EstadoTabuleiro,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for ControladorBatalha {
    fn init(base: Base<Node2D>) -> Self {
        let mut tabuleiro_jogador = EstadoTabuleiro::vazio();
        let mut tabuleiro_inimigo = EstadoTabuleiro::vazio();

        // No futuro, implementar a fase de preparação onde o jogador posiciona seus navios
        tabuleiro_jogador.preencher_aleatoriamente();
        tabuleiro_inimigo.preencher_aleatoriamente();

        Self {
            tabuleiro_jogador,
            tabuleiro_inimigo,
            base,
        }
    }

    fn ready(&mut self) {
        self.atualizar_visual_meu_campo();
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
            if mouse_event.is_pressed() && mouse_event.get_button_index() == MouseButton::LEFT {
                let click_pos = mouse_event.get_global_position();

                if let Some(mut enemy_map) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") {
                    let local_pos = enemy_map.to_local(click_pos);
                    let map_coord = enemy_map.local_to_map(local_pos);

                    if map_coord.x >= 0 && map_coord.x < BOARD_SIZE as i32 &&
                       map_coord.y >= 0 && map_coord.y < BOARD_SIZE as i32 {
                        
                        let x = map_coord.x as usize;
                        let y = map_coord.y as usize;
                        let retorno = executar_disparo(&mut self.tabuleiro_inimigo, x, y);

                        godot_print!("{}", retorno.mensagem);

                        match retorno.resultado {
                            ResultadoDisparo::Agua => {
                                enemy_map.set_cell_ex(map_coord)
                                    .source_id(0)
                                    .atlas_coords(Vector2i::new(8, 3))
                                    .done();
                            }
                            ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_) => {
                                enemy_map.set_cell_ex(map_coord)
                                    .source_id(0)
                                    .atlas_coords(Vector2i::new(10, 3))
                                    .done();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

impl ControladorBatalha {
    fn atualizar_visual_meu_campo(&mut self) {
        if let Some(mut player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            for x in 0..BOARD_SIZE {
                for y in 0..BOARD_SIZE {
                    if let Some(Celula::Ocupado(_)) = self.tabuleiro_jogador.valor_celula(x, y) {
                        player_map.set_cell_ex(Vector2i::new(x as i32, y as i32))
                            .source_id(0)
                            .atlas_coords(Vector2i::new(8, 7))
                            .done();
                    }
                }
            }
        }
    }
}