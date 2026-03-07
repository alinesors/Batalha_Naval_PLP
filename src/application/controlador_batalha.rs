use godot::classes::{
    INode2D, Input, InputEvent, InputEventMouseButton, Label, Node, Node2D, TileMapLayer,
};
use godot::global::MouseButton;
use godot::prelude::*;

use crate::application::fase_posicionamento::FasePosicionamento;
use crate::domain::disparo::ResultadoDisparo;
use crate::domain::jogador::Jogador;
use crate::domain::jogador_ia::JogadorIA;
use crate::domain::tabuleiro::{Celula, BOARD_SIZE};

const DELAY_TURNO_IA: f64 = 0.7;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FaseJogo {
    PosicionandoJogador,
    TurnoJogador,
    TurnoIAAguardandoDelay,
    FimDeJogo,
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct ControladorBatalha {
    jogador_humano: Jogador,
    jogador_ia: JogadorIA,
    fase_posicionamento: FasePosicionamento,
    fase: FaseJogo,
    tempo_restante_ia: f64,
    tooltip_instrucao: Option<Gd<Label>>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for ControladorBatalha {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            jogador_humano: Jogador::novo_humano(),
            jogador_ia: JogadorIA::novo_facil(),
            fase_posicionamento: FasePosicionamento::nova(),
            fase: FaseJogo::PosicionandoJogador,
            tempo_restante_ia: 0.0,
            tooltip_instrucao: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.criar_tooltip_instrucao();
        self.atualizar_visual_meu_campo();
        godot_print!(
            "Partida: {:?} vs {:?}",
            self.jogador_humano.tipo(),
            self.jogador_ia.tipo()
        );
    }

    fn process(&mut self, delta: f64) {
        self.atualizar_tooltip_posicionamento();

        if self.fase == FaseJogo::PosicionandoJogador {
            let input = Input::singleton();
            if input.is_action_just_pressed("rotacionar_navio") {
                self.fase_posicionamento.alternar_orientacao();
                godot_print!(
                    "Orientação alterada para {}.",
                    self.fase_posicionamento.orientacao_texto().to_lowercase()
                );
            }
        }

        if self.fase == FaseJogo::TurnoIAAguardandoDelay {
            self.tempo_restante_ia -= delta;
            if self.tempo_restante_ia <= 0.0 {
                self.executar_turno_ia();
            }
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if self.fase == FaseJogo::FimDeJogo {
            return;
        }

        if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
            if !mouse_event.is_pressed() || mouse_event.get_button_index() != MouseButton::LEFT {
                return;
            }

            let click_pos = mouse_event.get_global_position();

            if self.fase == FaseJogo::PosicionandoJogador {
                self.tratar_clique_posicionamento(click_pos);
                return;
            }

            if self.fase == FaseJogo::TurnoJogador {
                self.tratar_clique_disparo_jogador(click_pos);
            }
        }
    }
}

impl ControladorBatalha {
    fn criar_tooltip_instrucao(&mut self) {
        let mut tooltip = Label::new_alloc();
        tooltip.set_visible(false);
        tooltip.set_scale(Vector2::new(0.5, 0.5));
        self.base_mut().add_child(&tooltip.clone().upcast::<Node>());
        self.tooltip_instrucao = Some(tooltip);
    }

    fn atualizar_tooltip_posicionamento(&mut self) {
        let Some(mut tooltip) = self.tooltip_instrucao.clone() else {
            return;
        };

        if self.fase != FaseJogo::PosicionandoJogador {
            tooltip.set_visible(false);
            return;
        }

        let Some((nome, tamanho)) = self.fase_posicionamento.navio_atual() else {
            tooltip.set_visible(false);
            return;
        };

        let texto = format!(
            "Posicione: {} ({})\nClique: posicionar | R: rotacionar ({})",
            nome,
            tamanho,
            self.fase_posicionamento.orientacao_texto()
        );

        tooltip.set_text(&texto);
        tooltip.set_visible(true);

        let mouse_pos_global = self.base().get_global_mouse_position();
        let mouse_pos_local = self.base().to_local(mouse_pos_global);
        tooltip.set_position(mouse_pos_local + Vector2::new(14.0, 14.0));
    }

    fn tratar_clique_posicionamento(&mut self, click_pos: Vector2) {
        let Some(player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") else {
            return;
        };

        let Some((x, y, _)) = Self::coordenada_tabuleiro_do_clique(player_map, click_pos) else {
            return;
        };

        let nome_navio = match self.fase_posicionamento.navio_atual() {
            Some((nome, _)) => nome.to_string(),
            None => "Navio".to_string(),
        };

        match self
            .fase_posicionamento
            .tentar_posicionar_navio(&mut self.jogador_humano, x, y)
        {
            Ok(concluiu) => {
                self.atualizar_visual_meu_campo();
                if concluiu {
                    self.iniciar_fase_batalha();
                }
            }
            Err(erro) => {
                godot_print!("Não foi possível posicionar {}: {}", nome_navio, erro);
            }
        }
    }

    fn iniciar_fase_batalha(&mut self) {
        self.jogador_ia
            .jogador_mut()
            .tabuleiro_mut()
            .preencher_aleatoriamente();
        self.fase = FaseJogo::TurnoJogador;
        godot_print!("Frotas prontas. Batalha iniciada! O jogador começa atirando.");
    }

    fn tratar_clique_disparo_jogador(&mut self, click_pos: Vector2) {
        let Some(mut enemy_map) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") else {
            return;
        };

        let Some((x, y, map_coord)) =
            Self::coordenada_tabuleiro_do_clique(enemy_map.clone(), click_pos)
        else {
            return;
        };

        let retorno = self.jogador_ia.receber_disparo(x, y);
        godot_print!("{}", retorno.mensagem);

        self.atualizar_celula_visual_disparo(&mut enemy_map, map_coord, &retorno.resultado);

        if self.verificar_fim_de_jogo() {
            return;
        }

        if Self::disparo_foi_valido(&retorno.resultado) {
            self.fase = FaseJogo::TurnoIAAguardandoDelay;
            self.tempo_restante_ia = DELAY_TURNO_IA;
        }
    }

    fn executar_turno_ia(&mut self) {
        let Some((x, y)) = self
            .jogador_ia
            .escolher_alvo(self.jogador_humano.tabuleiro())
        else {
            self.fase = FaseJogo::FimDeJogo;
            godot_print!("Sem alvos restantes para a IA.");
            return;
        };

        let retorno = self.jogador_humano.receber_disparo(x, y);
        godot_print!("Turno da IA: {}", retorno.mensagem);

        if let Some(mut player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            self.atualizar_celula_visual_disparo(
                &mut player_map,
                Vector2i::new(x as i32, y as i32),
                &retorno.resultado,
            );
        }

        if self.verificar_fim_de_jogo() {
            return;
        }

        self.fase = FaseJogo::TurnoJogador;
    }

    fn verificar_fim_de_jogo(&mut self) -> bool {
        if self.jogador_ia.perdeu() {
            self.fase = FaseJogo::FimDeJogo;
            godot_print!("Fim de jogo! O jogador venceu.");
            return true;
        }

        if self.jogador_humano.perdeu() {
            self.fase = FaseJogo::FimDeJogo;
            godot_print!("Fim de jogo! A IA venceu.");
            return true;
        }

        false
    }

    fn disparo_foi_valido(resultado: &ResultadoDisparo) -> bool {
        matches!(
            resultado,
            ResultadoDisparo::Agua | ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_)
        )
    }

    fn coordenada_tabuleiro_do_clique(
        map: Gd<TileMapLayer>,
        click_pos: Vector2,
    ) -> Option<(usize, usize, Vector2i)> {
        let local_pos = map.to_local(click_pos);
        let map_coord = map.local_to_map(local_pos);

        if map_coord.x < 0
            || map_coord.y < 0
            || map_coord.x >= BOARD_SIZE as i32
            || map_coord.y >= BOARD_SIZE as i32
        {
            return None;
        }

        Some((map_coord.x as usize, map_coord.y as usize, map_coord))
    }

    fn atualizar_celula_visual_disparo(
        &self,
        map: &mut Gd<TileMapLayer>,
        map_coord: Vector2i,
        resultado: &ResultadoDisparo,
    ) {
        match resultado {
            ResultadoDisparo::Agua => {
                map.set_cell_ex(map_coord)
                    .source_id(0)
                    .atlas_coords(Vector2i::new(8, 3))
                    .done();
            }
            ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_) => {
                map.set_cell_ex(map_coord)
                    .source_id(0)
                    .atlas_coords(Vector2i::new(10, 3))
                    .done();
            }
            ResultadoDisparo::JaDisparado | ResultadoDisparo::ForaDosLimites => {}
        }
    }

    fn atualizar_visual_meu_campo(&mut self) {
        if let Some(mut player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            for x in 0..BOARD_SIZE {
                for y in 0..BOARD_SIZE {
                    let map_coord = Vector2i::new(x as i32, y as i32);
                    if let Some(celula) = self.jogador_humano.tabuleiro().valor_celula(x, y) {
                        match celula {
                            Celula::Ocupado(_) => {
                                player_map
                                    .set_cell_ex(map_coord)
                                    .source_id(0)
                                    .atlas_coords(Vector2i::new(8, 7))
                                    .done();
                            }
                            Celula::Agua => {
                                player_map
                                    .set_cell_ex(map_coord)
                                    .source_id(0)
                                    .atlas_coords(Vector2i::new(8, 3))
                                    .done();
                            }
                            Celula::Atingido(_) => {
                                player_map
                                    .set_cell_ex(map_coord)
                                    .source_id(0)
                                    .atlas_coords(Vector2i::new(10, 3))
                                    .done();
                            }
                            Celula::Vazio => {}
                        }
                    }
                }
            }
        }
    }
}
