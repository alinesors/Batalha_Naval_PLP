use godot::classes::{INode2D, Input, InputEvent, InputEventKey, InputEventMouseButton, Label, Node2D, TileMapLayer};
use godot::global::{Key, MouseButton};
use godot::prelude::*;

use crate::application::fase_posicionamento::FasePosicionamento;
use crate::application::fase_selecao_dificuldade::FaseSelecaoDificuldade;
use crate::application::gerenciador_audio::GerenciadorAudio;
use crate::application::gerenciador_interface::GerenciadorInterface;
use crate::application::gerenciador_turnos::{EstadoTurno, GerenciadorTurnos};
use crate::application::helpers::{conversao_coordenadas, coordenadas, cursor};
use crate::domain::disparo::ResultadoDisparo;
use crate::domain::jogador::Jogador;
use crate::domain::jogador_ia::JogadorIA;
use crate::presentation::batalha::{
    limpar_preview, render_preview_posicionamento, render_resultado_disparo, 
    render_navio_afundado, render_tabuleiro_jogador,
};

const DELAY_TURNO_IA: f64 = 1.0;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct ControladorBatalha {
    jogador_humano: Jogador,
    jogador_ia: Option<JogadorIA>,
    fase_posicionamento: FasePosicionamento,
    fase_selecao_dificuldade: FaseSelecaoDificuldade,
    gerenciador_turnos: GerenciadorTurnos,
    gerenciador_interface: GerenciadorInterface,
    gerenciador_audio: GerenciadorAudio,
    tempo_restante_ia: f64,
    estado_anterior: EstadoTurno,
    tooltip_instrucao: Option<Gd<Label>>,
    resultado_final_emitido: bool,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for ControladorBatalha {
    fn init(base: Base<Node2D>) -> Self {
        let total_navios: u32 = crate::domain::tabuleiro::FROTA_PADRAO
            .iter()
            .map(|config| config.quantidade as u32)
            .sum();

        Self {
            jogador_humano: Jogador::novo_humano(),
            jogador_ia: None,
            fase_posicionamento: FasePosicionamento::nova(),
            fase_selecao_dificuldade: FaseSelecaoDificuldade::nova(),
            gerenciador_turnos: GerenciadorTurnos::novo(total_navios),
            gerenciador_interface: GerenciadorInterface::novo(),
            gerenciador_audio: GerenciadorAudio::novo(),
            tempo_restante_ia: 0.0,
            estado_anterior: EstadoTurno::SelecaoDificuldade,
            tooltip_instrucao: None,
            resultado_final_emitido: false,
            base,
        }
    }

    fn ready(&mut self) {
        if let Some(campo_jogador) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            coordenadas::gerar_coordenadas(campo_jogador);
        }
        if let Some(campo_ia) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") {
            coordenadas::gerar_coordenadas(campo_ia);
        }

        self.gerenciador_interface.inicializar(self.base().clone());
        
        // Inicializar áudio com o nó base
        let node = self.base().clone().upcast::<Node>();
        self.gerenciador_audio.inicializar(&node);
        
        // Iniciar música e ondas desde o início (planejamento)
        self.gerenciador_audio.tocar_musica_batalha();
        self.gerenciador_audio.tocar_ondas();
    }

    fn process(&mut self, delta: f64) {
        // Atualizar interface primeiro, independente do estado
        self.gerenciador_interface.atualizar(
            self.gerenciador_turnos.estado_atual(),
            self.gerenciador_turnos.rodada_atual(),
        );

        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(campo_jogador) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
                cursor::esconder_cursor(campo_jogador);
            }
            if let Some(campo_ia) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") {
                cursor::esconder_cursor(campo_ia);
            }
            
            return;
        }

        self.atualizar_tooltip_posicionamento();
        
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::PosicionamentoJogador {
            self.atualizar_preview_posicionamento();
            let input = Input::singleton();
            if input.is_action_just_pressed("rotacionar_navio") {
                self.fase_posicionamento.alternar_orientacao();
            }
        } else {
            self.limpar_preview_posicionamento();
        }

        self.atualizar_controle_cursor();

        // Processar delays de som
        self.gerenciador_audio.processar_delays(delta);

        // Detectar fim de jogo e tocar sons apropriados
        let estado_atual = self.gerenciador_turnos.estado_atual();
        if estado_atual != self.estado_anterior {
            match estado_atual {
                EstadoTurno::VitoriaJogador => {
                    self.gerenciador_audio.tocar_vitoria();
                    self.emitir_resultado_final(true);
                }
                EstadoTurno::VitoriaIA => {
                    self.gerenciador_audio.tocar_derrota();
                    self.emitir_resultado_final(false);
                }
                _ => {}
            }
            self.estado_anterior = estado_atual;
        }

        if self.gerenciador_turnos.estado_atual() == EstadoTurno::TurnoIA {
            self.tempo_restante_ia -= delta;
            if self.tempo_restante_ia <= 0.0 {
                self.executar_turno_ia();
            }
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        // Detectar R para reiniciar quando o jogo terminou
        if self.gerenciador_turnos.jogo_terminou() {
            if let Ok(key_event) = event.try_cast::<InputEventKey>() {
                if key_event.is_pressed() && !key_event.is_echo() {
                    let keycode = key_event.get_keycode();
                    if keycode == Key::R {
                        let mut tree = self.base().get_tree();
                        tree.reload_current_scene();
                    }
                }
            }
            return;
        }

        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Ok(key_event) = event.try_cast::<InputEventKey>() {
                if key_event.is_pressed() && !key_event.is_echo() {
                    let keycode = key_event.get_keycode();
                    if let Some(ia) = self.fase_selecao_dificuldade.processar_tecla(keycode) {
                        self.jogador_ia = Some(ia);
                        self.gerenciador_turnos.confirmar_dificuldade();
                    }
                }
            }
            return;
        }
        
        if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
            if !mouse_event.is_pressed() || mouse_event.get_button_index() != MouseButton::LEFT {
                return;
            }
            let click_pos = self.base().get_global_mouse_position();
            
            match self.gerenciador_turnos.estado_atual() {
                EstadoTurno::PosicionamentoJogador => {
                    self.tratar_clique_posicionamento(click_pos);
                }
                EstadoTurno::TurnoJogador => {
                    self.tratar_clique_disparo_jogador(click_pos);
                }
                _ => {}
            }
        }
    }
}

#[godot_api]
impl ControladorBatalha {
    #[signal]
    fn batalha_encerrada(vitoria: bool);

    #[func]
    pub fn selecionar_dificuldade_facil(&mut self) {
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(ia) = self.fase_selecao_dificuldade.processar_selecao(0) {
                self.jogador_ia = Some(ia);
                self.gerenciador_turnos.confirmar_dificuldade();
            }
        }
    }

    #[func]
    pub fn selecionar_dificuldade_media(&mut self) {
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(ia) = self.fase_selecao_dificuldade.processar_selecao(1) {
                self.jogador_ia = Some(ia);
                self.gerenciador_turnos.confirmar_dificuldade();
            }
        }
    }

    #[func]
    pub fn selecionar_dificuldade_dificil(&mut self) {
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(ia) = self.fase_selecao_dificuldade.processar_selecao(2) {
                self.jogador_ia = Some(ia);
                self.gerenciador_turnos.confirmar_dificuldade();
            }
        }
    }

    #[func]
    pub fn vencer_teste(&mut self) {
        if self.gerenciador_turnos.jogo_terminou() {
            return;
        }

        self.gerenciador_turnos.forcar_vitoria_jogador();
    }

    #[func]
    pub fn perder_teste(&mut self) {
        if self.gerenciador_turnos.jogo_terminou() {
            return;
        }

        self.gerenciador_turnos.forcar_vitoria_ia();
    }

    #[func]
    pub fn confirmar_posicionamento(&mut self) {
        if self.fase_posicionamento.em_modo_edicao() 
            && self.fase_posicionamento.todos_posicionados() {
            self.gerenciador_interface.esconder_botao_confirmar();
            self.iniciar_fase_batalha();
        }
    }
}

impl ControladorBatalha {
    fn emitir_resultado_final(&mut self, vitoria: bool) {
        if self.resultado_final_emitido {
            return;
        }

        self.resultado_final_emitido = true;
        self.base_mut()
            .emit_signal("batalha_encerrada", &[vitoria.to_variant()]);
    }

    fn atualizar_tooltip_posicionamento(&mut self) {
        let Some(mut tooltip) = self.tooltip_instrucao.clone() else {
            return;
        };

        if self.gerenciador_turnos.estado_atual() != EstadoTurno::PosicionamentoJogador {
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

        let Some((x, y, _)) = conversao_coordenadas::clique_para_coordenada(player_map, click_pos) else {
            return;
        };

        // Se está em modo edição, tentar remover navio
        if self.fase_posicionamento.em_modo_edicao() {
            if let Some(nome_navio) = self.jogador_humano.tabuleiro_mut().remover_navio_na_posicao(x, y) {
                if self.fase_posicionamento.remover_navio(&nome_navio) {
                    self.atualizar_visual_meu_campo();
                    self.gerenciador_interface.esconder_botao_confirmar();
                }
            }
            return;
        }

        // Tentar posicionar novo navio
        match self
            .fase_posicionamento
            .tentar_posicionar_navio(&mut self.jogador_humano, x, y)
        {
            Ok(concluiu) => {
                self.atualizar_visual_meu_campo();
                if concluiu {
                    // Entrar em modo edição ao invés de começar imediatamente
                    self.fase_posicionamento.ativar_modo_edicao();
                    self.gerenciador_interface.mostrar_botao_confirmar();
                }
            }
            Err(_) => {}
        }
    }

    fn iniciar_fase_batalha(&mut self) {
        self.gerenciador_turnos.finalizar_posicionamento_jogador();
        
        if let Some(ref mut ia) = self.jogador_ia {
            ia.jogador_mut().tabuleiro_mut().preencher_aleatoriamente();
        }
        
        self.limpar_preview_posicionamento();
        self.gerenciador_turnos.iniciar_jogo();
    }

    fn atualizar_preview_posicionamento(&mut self) {
        let Some(player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") else {
            return;
        };
        let Some(mut preview_map) = self
            .base()
            .try_get_node_as::<TileMapLayer>("PreviewPosicionamento")
        else {
            return;
        };

        let mouse_pos = self.base().get_global_mouse_position();
        let Some((x, y, _)) = conversao_coordenadas::clique_para_coordenada(player_map, mouse_pos) else {
            limpar_preview(&mut preview_map);
            return;
        };

        let Some((nome_navio, _)) = self.fase_posicionamento.navio_atual() else {
            limpar_preview(&mut preview_map);
            return;
        };

        let Some(preview) = self
            .fase_posicionamento
            .preview_na_posicao(&self.jogador_humano, x, y)
        else {
            limpar_preview(&mut preview_map);
            return;
        };
        
        render_preview_posicionamento(&mut preview_map, nome_navio, &preview.celulas, preview.valido);
    }

    fn limpar_preview_posicionamento(&mut self) {
        if let Some(mut preview_map) = self
            .base()
            .try_get_node_as::<TileMapLayer>("PreviewPosicionamento")
        {
            limpar_preview(&mut preview_map);
        }
    }

    fn tratar_clique_disparo_jogador(&mut self, click_pos: Vector2) {
        let Some(mut enemy_map) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") else {
            return;
        };

        let Some((x, y, map_coord)) =
            conversao_coordenadas::clique_para_coordenada(enemy_map.clone(), click_pos)
        else {
            return;
        };

        let (retorno, ia_perdeu) = {
            let Some(ref mut ia) = self.jogador_ia else {
                return;
            };
            let retorno = ia.receber_disparo(x, y);
            godot_print!("{}", retorno.mensagem);
            let ia_perdeu = ia.perdeu();
            (retorno, ia_perdeu)
        };

        render_resultado_disparo(&mut enemy_map, map_coord, &retorno.resultado);
        
        // Se um navio afundou, renderizar todas as suas células como afundadas
        if let ResultadoDisparo::Afundou(_) = &retorno.resultado {
            if let Some(ref ia) = self.jogador_ia {
                // Encontrar o índice do navio que afundou
                for (idx, navio) in ia.tabuleiro().navios.iter().enumerate() {
                    if navio.esta_afundado() {
                        render_navio_afundado(&mut enemy_map, ia.tabuleiro(), idx);
                    }
                }
            }
        }

        if retorno.resultado.foi_valido() {
            // Só tocar som se o disparo foi válido
            self.gerenciador_audio.tocar_disparo_com_resultado(&retorno.resultado);
            
            let acertou = matches!(retorno.resultado, ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_));
            let afundou = matches!(retorno.resultado, ResultadoDisparo::Afundou(_));
            
            self.gerenciador_turnos.processar_ataque_jogador(acertou, afundou);
            
            if ia_perdeu {
                return;
            }
            
            if !acertou && !self.gerenciador_turnos.jogo_terminou() {
                self.tempo_restante_ia = DELAY_TURNO_IA;
            }
        }
    }

    fn executar_turno_ia(&mut self) {
        let (x, y, retorno) = {
            let Some(ref mut ia) = self.jogador_ia else {
                return;
            };

            let Some((x, y)) = ia.escolher_alvo(self.jogador_humano.tabuleiro()) else {
                return;
            };

            let retorno = self.jogador_humano.receber_disparo(x, y);
            godot_print!("IA: {}", retorno.mensagem);

            (x, y, retorno)
        };

        // Tocar disparo e agendar resultado
        self.gerenciador_audio.tocar_disparo_com_resultado(&retorno.resultado);

        // Notificar a IA do resultado
        if let Some(ref mut ia) = self.jogador_ia {
            ia.notificar_resultado(x, y, &retorno);
        }

        if let Some(mut player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            render_resultado_disparo(
                &mut player_map,
                Vector2i::new(y as i32, x as i32),
                &retorno.resultado,
            );
            
            // Se um navio afundou, renderizar todas as suas células como afundadas
            if let ResultadoDisparo::Afundou(_) = &retorno.resultado {
                // Encontrar o índice do navio que afundou
                for (idx, navio) in self.jogador_humano.tabuleiro().navios.iter().enumerate() {
                    if navio.esta_afundado() {
                        render_navio_afundado(&mut player_map, self.jogador_humano.tabuleiro(), idx);
                    }
                }
            }
        }

        let acertou = matches!(retorno.resultado, ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_));
        let afundou = matches!(retorno.resultado, ResultadoDisparo::Afundou(_));
        
        self.gerenciador_turnos.processar_ataque_ia(acertou, afundou);

        if acertou && self.gerenciador_turnos.estado_atual() == EstadoTurno::TurnoIA {
            self.tempo_restante_ia = DELAY_TURNO_IA;
        }
    }

    fn atualizar_controle_cursor(&mut self) {
        let mouse_pos = self.base().get_global_mouse_position();
        let estado = self.gerenciador_turnos.estado_atual();

        let (mostrar_jogador, mostrar_ia) = match estado {
            EstadoTurno::PosicionamentoJogador => (true, false),
            EstadoTurno::TurnoJogador => (false, true),
            _ => (false, false),
        };

        if let Some(campo_jogador) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            if mostrar_jogador {
                cursor::controlar_cursor(campo_jogador, mouse_pos);
            } else {
                cursor::esconder_cursor(campo_jogador);
            }
        }

        if let Some(campo_ia) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") {
            if mostrar_ia {
                cursor::controlar_cursor(campo_ia, mouse_pos);
            } else {
                cursor::esconder_cursor(campo_ia);
            }
        }
    }

    fn atualizar_visual_meu_campo(&mut self) {
        if let Some(mut player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            render_tabuleiro_jogador(&mut player_map, self.jogador_humano.tabuleiro());
        }
    }
}
