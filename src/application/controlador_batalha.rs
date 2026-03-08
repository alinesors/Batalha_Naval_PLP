use godot::classes::{INode2D, Input, InputEvent, InputEventKey, InputEventMouseButton, Label, Node2D, TileMapLayer};
use godot::global::MouseButton;
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
    vitoria_registrada: Option<bool>,
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
            vitoria_registrada: None,
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

        // Detectar mudança de estado para PosicionamentoJogador e popular container
        if self.estado_anterior == EstadoTurno::SelecaoDificuldade && 
           self.gerenciador_turnos.estado_atual() == EstadoTurno::PosicionamentoJogador {
            godot_print!("Mudou para PosicionamentoJogador - populando container");
            self.popular_container_navios();
        }
        self.estado_anterior = self.gerenciador_turnos.estado_atual();

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
                    self.gerenciador_interface.atualizar(estado_atual, self.gerenciador_turnos.rodada_atual());
                }
                EstadoTurno::VitoriaIA => {
                    self.gerenciador_audio.tocar_derrota();
                    self.emitir_resultado_final(false);
                    self.gerenciador_interface.atualizar(estado_atual, self.gerenciador_turnos.rodada_atual());
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
        // Game over state is handled by the Continuar button
        if self.gerenciador_turnos.jogo_terminou() {
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
        godot_print!("selecionar_dificuldade_facil chamado");
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(ia) = self.fase_selecao_dificuldade.processar_selecao(0) {
                self.jogador_ia = Some(ia);
                self.gerenciador_turnos.confirmar_dificuldade();
                godot_print!("Dificuldade confirmada, estado agora: {:?}", self.gerenciador_turnos.estado_atual());
            }
        }
    }

    #[func]
    pub fn selecionar_dificuldade_media(&mut self) {
        godot_print!("selecionar_dificuldade_media chamado");
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(ia) = self.fase_selecao_dificuldade.processar_selecao(1) {
                self.jogador_ia = Some(ia);
                self.gerenciador_turnos.confirmar_dificuldade();
                godot_print!("Dificuldade confirmada, estado agora: {:?}", self.gerenciador_turnos.estado_atual());
            }
        }
    }

    #[func]
    pub fn selecionar_dificuldade_dificil(&mut self) {
        godot_print!("selecionar_dificuldade_dificil chamado");
        if self.gerenciador_turnos.estado_atual() == EstadoTurno::SelecaoDificuldade {
            if let Some(ia) = self.fase_selecao_dificuldade.processar_selecao(2) {
                self.jogador_ia = Some(ia);
                self.gerenciador_turnos.confirmar_dificuldade();
                godot_print!("Dificuldade confirmada, estado agora: {:?}", self.gerenciador_turnos.estado_atual());
            }
        }
    }

    #[func]
    pub fn vencer_teste(&mut self) {
        if self.gerenciador_turnos.jogo_terminou() {
            return;
        }

        self.gerenciador_turnos.forcar_vitoria_jogador();
        self.gerenciador_audio.tocar_vitoria();
        self.emitir_resultado_final(true);
        self.gerenciador_interface.atualizar(
            EstadoTurno::VitoriaJogador, 
            self.gerenciador_turnos.rodada_atual()
        );
        self.estado_anterior = EstadoTurno::VitoriaJogador;
    }

    #[func]
    pub fn perder_teste(&mut self) {
        if self.gerenciador_turnos.jogo_terminou() {
            return;
        }

        self.gerenciador_turnos.forcar_vitoria_ia();
        self.gerenciador_audio.tocar_derrota();
        self.emitir_resultado_final(false);
        self.gerenciador_interface.atualizar(
            EstadoTurno::VitoriaIA, 
            self.gerenciador_turnos.rodada_atual()
        );
        self.estado_anterior = EstadoTurno::VitoriaIA;
    }

    #[func]
    pub fn confirmar_posicionamento(&mut self) {
        if self.fase_posicionamento.em_modo_edicao() 
            && self.fase_posicionamento.todos_posicionados() {
            self.gerenciador_interface.esconder_botao_confirmar();
            self.iniciar_fase_batalha();
        }
    }

    #[func]
    pub fn continuar(&mut self) {
        if let Some(vitoria) = self.vitoria_registrada {
            self.base_mut()
                .emit_signal("batalha_encerrada", &[vitoria.to_variant()]);
        }
    }

    #[func]
    pub fn selecionar_navio_do_container(&mut self, indice: i32) {
        if indice < 0 {
            return;
        }
        
        if self.fase_posicionamento.selecionar_navio(indice as usize) {
            godot_print!("Navio {} selecionado", indice);
        }
    }
}

impl ControladorBatalha {
    fn emitir_resultado_final(&mut self, vitoria: bool) {
        if self.resultado_final_emitido {
            return;
        }

        self.resultado_final_emitido = true;
        self.vitoria_registrada = Some(vitoria);
        // Signal will be emitted by continuar() method when button is pressed
    }

    fn popular_container_navios(&mut self) {
        let Some(mut container) = self.gerenciador_interface.container_navios() else {
            godot_print!("ERRO: Container de navios não encontrado!");
            return;
        };

        use godot::classes::{AtlasTexture, Button, FontFile, HBoxContainer as GdHBoxContainer, 
                             ResourceLoader, Texture2D, TextureRect, VBoxContainer};
        use godot::classes::box_container::AlignmentMode;
        
        // Limpar container primeiro
        for mut child in container.get_children().iter_shared() {
            child.queue_free();
        }

        let fila_navios = self.fase_posicionamento.obter_fila_navios();
        godot_print!("Popular container com {} navios", fila_navios.len());
        
        // Carregar recursos
        let mut resource_loader = ResourceLoader::singleton();
        let font = resource_loader
            .load("res://fonts/Retro Gaming.ttf")
            .and_then(|res| res.try_cast::<FontFile>().ok());
        
        let textura_navios = resource_loader
            .load("res://textures/Water+.png")
            .and_then(|res| res.try_cast::<Texture2D>().ok());

        for (idx, (nome, tamanho)) in fila_navios.iter().enumerate() {
            // Container vertical para cada navio (sprites + botão)
            let mut vbox = VBoxContainer::new_alloc();
            vbox.set_custom_minimum_size(Vector2::new((*tamanho as f32) * 12.0 + 8.0, 40.0));
            
            // Container horizontal para os sprites
            let mut hbox_sprites = GdHBoxContainer::new_alloc();
            hbox_sprites.set_alignment(AlignmentMode::CENTER); // Centralizado
            hbox_sprites.add_theme_constant_override("separation", 1);
            
            // Criar sprites do navio (repetir o sprite N vezes baseado no tamanho)
            if let Some(ref textura) = textura_navios {
                for _ in 0..*tamanho {
                    let mut atlas = AtlasTexture::new_gd();
                    atlas.set_atlas(textura);
                    // Sprite do navio em (8, 7) no atlas - cada tile é 16x16
                    atlas.set_region(Rect2::new(Vector2::new(8.0 * 16.0, 7.0 * 16.0), Vector2::new(16.0, 16.0)));
                    
                    let mut sprite_rect = TextureRect::new_alloc();
                    sprite_rect.set_texture(&atlas.upcast::<Texture2D>());
                    sprite_rect.set_custom_minimum_size(Vector2::new(12.0, 12.0));
                    sprite_rect.set_expand_mode(godot::classes::texture_rect::ExpandMode::IGNORE_SIZE);
                    sprite_rect.set_stretch_mode(godot::classes::texture_rect::StretchMode::KEEP);
                    
                    hbox_sprites.add_child(&sprite_rect);
                }
            }
            
            vbox.add_child(&hbox_sprites);
            
            // Botão clicável embaixo dos sprites
            let mut botao = Button::new_alloc();
            if let Some(ref font_file) = font {
                botao.add_theme_font_override("font", font_file);
            }
            botao.add_theme_font_size_override("font_size", 8);
            botao.set_text(nome);
            botao.set_custom_minimum_size(Vector2::new((*tamanho as f32) * 12.0 + 8.0, 20.0));
            
            // Conectar sinal
            let controlador = self.base().clone();
            let indice = idx as i32;
            botao.connect("pressed", &controlador.callable("selecionar_navio_do_container").bind(&[indice.to_variant()]));
            
            vbox.add_child(&botao);
            container.add_child(&vbox);
            
            godot_print!("Adicionado navio visual: {} com {} sprites", nome, tamanho);
        }
        
        container.set_visible(true);
        godot_print!("Container visível: {}", container.is_visible());
    }

    fn atualizar_container_navios(&mut self) {
        self.popular_container_navios();
        
        // Se não há mais navios, esconder container e mostrar botão
        if self.fase_posicionamento.todos_posicionados() {
            self.gerenciador_interface.esconder_container_navios();
            self.fase_posicionamento.ativar_modo_edicao();
            self.gerenciador_interface.mostrar_botao_confirmar();
        }
    }

    fn atualizar_tooltip_posicionamento(&mut self) {
        let Some(mut tooltip) = self.tooltip_instrucao.clone() else {
            return;
        };

        if self.gerenciador_turnos.estado_atual() != EstadoTurno::PosicionamentoJogador {
            tooltip.set_visible(false);
            return;
        }

        let texto = match self.fase_posicionamento.navio_atual() {
            Some((nome, tamanho)) => {
                format!(
                    "Posicione: {} ({})\nClique: posicionar | R: rotacionar ({})",
                    nome,
                    tamanho,
                    self.fase_posicionamento.orientacao_texto()
                )
            }
            None => {
                "Selecione um navio da lista abaixo para posicionar".to_string()
            }
        };

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

        // Primeiro, tentar remover navio existente na posição
        if let Some(nome_navio) = self.jogador_humano.tabuleiro_mut().remover_navio_na_posicao(x, y) {
            if self.fase_posicionamento.remover_navio(&nome_navio) {
                self.atualizar_visual_meu_campo();
                self.gerenciador_interface.esconder_botao_confirmar();
                // Navio removido fica selecionado para reposicionamento
                // Não faz return - deixa continuar para posicionar se clicar novamente
            }
            return;
        }

        // Se não havia navio na posição, tentar posicionar o navio selecionado
        // Tentar posicionar novo navio
        match self
            .fase_posicionamento
            .tentar_posicionar_navio(&mut self.jogador_humano, x, y)
        {
            Ok(_) => {
                self.atualizar_visual_meu_campo();
                self.atualizar_container_navios();

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
