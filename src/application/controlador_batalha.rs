use godot::classes::{Button, INode2D, Input, InputEvent, InputEventKey, InputEventMouseButton, Label, Node, Node2D, TileMapLayer};
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
use crate::domain::tabuleiro::{Celula, BOARD_SIZE};
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
    botao_xray: Option<Gd<Button>>,
    mapa_xray_ia: Option<Gd<TileMapLayer>>,
    mapa_xray_ia_navios: Option<Gd<TileMapLayer>>,
    label_xray_ia: Option<Gd<Label>>,
    mapa_navios_jogador: Option<Gd<TileMapLayer>>,
    mapa_navios_ia: Option<Gd<TileMapLayer>>,
    xray_ativo: bool,
    modo_dinamico: bool,
    navio_selecionado_movimento: Option<usize>,
    movimento_jogador_realizado: bool,
    tiros_jogador_no_tabuleiro_ia: [[bool; BOARD_SIZE]; BOARD_SIZE],
    tiros_ia_no_tabuleiro_jogador: [[bool; BOARD_SIZE]; BOARD_SIZE],
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
            botao_xray: None,
            mapa_xray_ia: None,
            mapa_xray_ia_navios: None,
            label_xray_ia: None,
            mapa_navios_jogador: None,
            mapa_navios_ia: None,
            xray_ativo: false,
            modo_dinamico: false,
            navio_selecionado_movimento: None,
            movimento_jogador_realizado: false,
            tiros_jogador_no_tabuleiro_ia: [[false; BOARD_SIZE]; BOARD_SIZE],
            tiros_ia_no_tabuleiro_jogador: [[false; BOARD_SIZE]; BOARD_SIZE],
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
        self.botao_xray = self.base().try_get_node_as::<Button>("BotaoXRay");
        self.atualizar_texto_botao_xray();
        self.inicializar_xray_ia();
        self.inicializar_ship_layers();

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
        } else if self.modo_dinamico
            && self.gerenciador_turnos.estado_atual() == EstadoTurno::TurnoJogador
            && !self.movimento_jogador_realizado
            && self.navio_selecionado_movimento.is_some()
        {
            self.atualizar_preview_movimento_dinamico();
        } else {
            self.limpar_preview_posicionamento();
        }

        self.atualizar_controle_cursor();
        self.atualizar_xray_ia();

        // Processar delays de som
        self.gerenciador_audio.processar_delays(delta);

        // Detectar fim de jogo e tocar sons apropriados
        let estado_atual = self.gerenciador_turnos.estado_atual();
        if estado_atual != self.estado_anterior {
            match estado_atual {
                EstadoTurno::TurnoJogador => {
                    self.movimento_jogador_realizado = false;
                }
                EstadoTurno::TurnoIA => {}
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
                        let mut ia = ia;
                        ia.configurar_modo_dinamico(self.modo_dinamico);
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
                    if self.modo_dinamico && !self.movimento_jogador_realizado {
                        if self.tratar_clique_movimento_jogador(click_pos) {
                            return;
                        }
                    }
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
                let mut ia = ia;
                ia.configurar_modo_dinamico(self.modo_dinamico);
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
                let mut ia = ia;
                ia.configurar_modo_dinamico(self.modo_dinamico);
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
                let mut ia = ia;
                ia.configurar_modo_dinamico(self.modo_dinamico);
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
    pub fn definir_modo_dinamico(&mut self, ativo: bool) {
        self.modo_dinamico = ativo;
        if !ativo {
            self.xray_ativo = false;
        }
        self.atualizar_texto_botao_xray();
        self.navio_selecionado_movimento = None;
        self.movimento_jogador_realizado = false;
        if let Some(ref mut ia) = self.jogador_ia {
            ia.configurar_modo_dinamico(ativo);
        }
    }

    #[func]
    pub fn alternar_xray(&mut self) {
        self.xray_ativo = !self.xray_ativo;
        self.atualizar_texto_botao_xray();
        self.atualizar_xray_ia();
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
    fn inicializar_xray_ia(&mut self) {
        let Some(campo_ia) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") else {
            return;
        };
        let tile_set = campo_ia.get_tile_set();

        let mut mapa = TileMapLayer::new_alloc();
        mapa.set_name("CampoIAXRay");
        mapa.set_z_index(120);
        mapa.set_scale(Vector2::new(0.55, 0.55));
        if let Some(ts) = tile_set.clone() {
            mapa.set_tile_set(&ts);
        }
        self.base_mut().add_child(&mapa.clone().upcast::<Node>());
        self.mapa_xray_ia = Some(mapa);

        // Ship layer do xray — fica acima do board xray
        let mut mapa_navios = TileMapLayer::new_alloc();
        mapa_navios.set_name("CampoIAXRayNavios");
        mapa_navios.set_z_index(121);
        mapa_navios.set_scale(Vector2::new(0.55, 0.55));
        if let Some(ts) = tile_set {
            mapa_navios.set_tile_set(&ts);
        }
        self.base_mut().add_child(&mapa_navios.clone().upcast::<Node>());
        self.mapa_xray_ia_navios = Some(mapa_navios);

        let mut label = Label::new_alloc();
        label.set_name("LabelXRayIA");
        label.set_text("X-RAY IA");
        label.add_theme_font_size_override("font_size", 12);
        label.add_theme_color_override("font_color", Color::from_rgb(1.0, 0.92, 0.25));
        label.add_theme_color_override("font_outline_color", Color::from_rgb(0.0, 0.0, 0.0));
        label.add_theme_constant_override("outline_size", 2);
        label.set_z_index(122);
        self.base_mut().add_child(&label.clone().upcast::<Node>());
        self.label_xray_ia = Some(label);
    }

    fn inicializar_ship_layers(&mut self) {
        if let Some(campo_jogador) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
            let tile_set = campo_jogador.get_tile_set();
            let z = campo_jogador.get_z_index();
            let pos = campo_jogador.get_position();
            let scale = campo_jogador.get_scale();

            let mut mapa = TileMapLayer::new_alloc();
            mapa.set_name("NaviosJogador");
            mapa.set_z_index(z + 1);
            mapa.set_position(pos);
            mapa.set_scale(scale);
            if let Some(ts) = tile_set {
                mapa.set_tile_set(&ts);
            }
            self.base_mut().add_child(&mapa.clone().upcast::<Node>());
            self.mapa_navios_jogador = Some(mapa);
        }

        if let Some(campo_ia) = self.base().try_get_node_as::<TileMapLayer>("CampoIA") {
            let tile_set = campo_ia.get_tile_set();
            let z = campo_ia.get_z_index();
            let pos = campo_ia.get_position();
            let scale = campo_ia.get_scale();

            let mut mapa = TileMapLayer::new_alloc();
            mapa.set_name("NaviosIA");
            mapa.set_z_index(z + 1);
            mapa.set_position(pos);
            mapa.set_scale(scale);
            if let Some(ts) = tile_set {
                mapa.set_tile_set(&ts);
            }
            self.base_mut().add_child(&mapa.clone().upcast::<Node>());
            self.mapa_navios_ia = Some(mapa);
        }
    }

    fn atualizar_texto_botao_xray(&mut self) {
        if let Some(ref mut botao) = self.botao_xray {
            let txt = if self.xray_ativo {
                "X-Ray: ON"
            } else {
                "X-Ray: OFF"
            };
            botao.set_text(txt);
        }
    }

    fn atualizar_xray_ia(&mut self) {
        let viewport = self.base().get_viewport_rect().size;
        let pos_x = 24.0;
        let pos_y = (viewport.y - 112.0).max(16.0);

        let mostrar_botao = self.modo_dinamico && !self.gerenciador_turnos.jogo_terminou();
        if let Some(ref mut botao) = self.botao_xray {
            botao.set_visible(mostrar_botao);
        }

        let Some(ref mut mapa_xray) = self.mapa_xray_ia else {
            return;
        };
        let mostrar = mostrar_botao && self.xray_ativo;
        mapa_xray.set_visible(mostrar);

        if let Some(ref mut label) = self.label_xray_ia {
            label.set_visible(mostrar);
        }
        if let Some(ref mut ship_xray) = self.mapa_xray_ia_navios {
            ship_xray.set_visible(mostrar);
        }
        if !mostrar {
            return;
        }

        mapa_xray.set_position(Vector2::new(pos_x, pos_y));
        if let Some(ref mut label) = self.label_xray_ia {
            label.set_position(Vector2::new(pos_x, pos_y - 20.0));
        }

        let Some(ref ia) = self.jogador_ia else {
            return;
        };
        if let Some(ref mut ship_xray) = self.mapa_xray_ia_navios {
            ship_xray.set_visible(true);
            ship_xray.set_position(Vector2::new(pos_x, pos_y));
            render_tabuleiro_jogador(mapa_xray, ship_xray, ia.tabuleiro());
        }
    }

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

        use godot::classes::{AtlasTexture, Button, Control, FontFile,
                             ResourceLoader, Sprite2D, Texture2D, VBoxContainer};

        // Limpar container primeiro
        for mut child in container.get_children().iter_shared() {
            child.queue_free();
        }

        let fila_navios = self.fase_posicionamento.obter_fila_navios();
        godot_print!("Popular container com {} navios", fila_navios.len());

        let mut resource_loader = ResourceLoader::singleton();
        let font = resource_loader
            .load("res://fonts/Retro Gaming.ttf")
            .and_then(|res| res.try_cast::<FontFile>().ok());

        // Sprite-sheet de navios (3 colunas × 14 linhas, tiles 16×16).
        let textura_navios = resource_loader
            .load("res://textures/ships.png")
            .and_then(|res| res.try_cast::<Texture2D>().ok());

        let base_row = |tam: usize| -> f32 {
            match tam { 1 => 0.0, 3 => 1.0, 4 => 4.0, 6 => 8.0, _ => 0.0 }
        };

        // Tamanho do tile exibido no container (px)
        const TILE: f32 = 12.0;

        for (idx, (nome, tamanho)) in fila_navios.iter().enumerate() {
            let tam = *tamanho;
            let ship_w = tam as f32 * TILE;

            let mut vbox = VBoxContainer::new_alloc();
            vbox.set_custom_minimum_size(Vector2::new(ship_w + 8.0, 40.0));

            let mut canvas = Control::new_alloc();
            canvas.set_custom_minimum_size(Vector2::new(ship_w, TILE));

            if let Some(ref textura) = textura_navios {
                let row_base = base_row(tam);
                for seg in 0..tam {
                    let mut atlas = AtlasTexture::new_gd();
                    atlas.set_atlas(textura);
                    let row = row_base + seg as f32;
                    atlas.set_region(Rect2::new(
                        Vector2::new(0.0, row * 16.0),
                        Vector2::new(16.0, 16.0),
                    ));

                    let mut sprite = Sprite2D::new_alloc();
                    sprite.set_texture(&atlas.upcast::<Texture2D>());
                    // Escalar de 16px para TILE px
                    let escala = TILE / 16.0;
                    sprite.set_scale(Vector2::new(escala, escala));
                    // Centralizar
                    sprite.set_position(Vector2::new(
                        seg as f32 * TILE + TILE / 2.0,
                        TILE / 2.0,
                    ));
                    // Girar
                    if tam > 1 {
                        sprite.set_rotation_degrees(-90.0);
                    }

                    canvas.add_child(&sprite.upcast::<godot::classes::Node>());
                }
            }

            vbox.add_child(&canvas.upcast::<godot::classes::Node>());
            
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
        
        self.movimento_jogador_realizado = false;
        self.navio_selecionado_movimento = None;
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
        if retorno.resultado.foi_valido() {
            self.tiros_jogador_no_tabuleiro_ia[x][y] = true;
        }

        // Se um navio afundou, revelar sprites no ship layer da IA
        if let ResultadoDisparo::Afundou(_) = &retorno.resultado {
            if let Some(ref ia) = self.jogador_ia {
                if let Some(ref mut ship_ia) = self.mapa_navios_ia {
                    for (idx, navio) in ia.tabuleiro().navios.iter().enumerate() {
                        if navio.esta_afundado() {
                            render_navio_afundado(ship_ia, ia.tabuleiro(), idx);
                        }
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

            self.movimento_jogador_realizado = false;
            self.navio_selecionado_movimento = None;
            self.limpar_preview_posicionamento();
            
            if !acertou && !self.gerenciador_turnos.jogo_terminou() {
                self.tempo_restante_ia = DELAY_TURNO_IA;
            }
        }
    }

    fn executar_turno_ia(&mut self) {
        if self.modo_dinamico {
            self.executar_movimento_ia_dinamico();
        }

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

        if retorno.resultado.foi_valido() {
            self.tiros_ia_no_tabuleiro_jogador[x][y] = true;
        }

        if matches!(retorno.resultado, ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_)) {
            if let Some(player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") {
                let map_coord = Vector2i::new(y as i32, x as i32);
            }
        }

        // Redesenha o tabuleiro do jogador com board_map + ship_map separados.
        if let (Some(mut board_map), Some(mut ship_map)) = (
            self.base().try_get_node_as::<TileMapLayer>("CampoJogador"),
            self.mapa_navios_jogador.clone(),
        ) {
            render_tabuleiro_jogador(&mut board_map, &mut ship_map, self.jogador_humano.tabuleiro());
        }

        let acertou = matches!(retorno.resultado, ResultadoDisparo::Acerto | ResultadoDisparo::Afundou(_));
        let afundou = matches!(retorno.resultado, ResultadoDisparo::Afundou(_));
        
        self.gerenciador_turnos.processar_ataque_ia(acertou, afundou);

        if acertou && self.gerenciador_turnos.estado_atual() == EstadoTurno::TurnoIA {
            self.tempo_restante_ia = DELAY_TURNO_IA;
        }
    }

    fn tratar_clique_movimento_jogador(&mut self, click_pos: Vector2) -> bool {
        let Some(player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") else {
            return false;
        };
        let Some((x, y, map_coord)) =
            conversao_coordenadas::clique_para_coordenada(player_map.clone(), click_pos)
        else {
            return false;
        };

        let tabuleiro = self.jogador_humano.tabuleiro();
        let celula = tabuleiro.valor_celula(x, y);

        if self.navio_selecionado_movimento.is_none() {
            let Some(celula) = celula else {
                return true;
            };
            let idx = match celula {
                Celula::Ocupado(idx) => idx,
                _ => return true,
            };
            if tabuleiro.navios.get(idx).is_some_and(|n| n.acertos == 0 && !n.esta_afundado()) {
                self.navio_selecionado_movimento = Some(idx);
            }
            return true;
        }

        if let Some(celula) = celula {
            if let Celula::Ocupado(idx) = celula {
                if tabuleiro.navios.get(idx).is_some_and(|n| n.acertos == 0 && !n.esta_afundado())
                {
                    self.navio_selecionado_movimento = Some(idx);
                }
                return true;
            }
        }

        let Some(navio_idx) = self.navio_selecionado_movimento else {
            return true;
        };
        let Some((dx, dy)) = self.inferir_direcao_movimento_por_clique(navio_idx, map_coord) else {
            return true;
        };

        if self
            .jogador_humano
            .tabuleiro()
            .pode_mover_navio(navio_idx, dx, dy)
        {
            let _ = self.jogador_humano.tabuleiro_mut().mover_navio(navio_idx, dx, dy);
            self.movimento_jogador_realizado = true;
            self.navio_selecionado_movimento = None;
            self.limpar_preview_posicionamento();
            self.atualizar_visual_meu_campo();
        }

        true
    }

    fn inferir_direcao_movimento_por_clique(
        &self,
        navio_idx: usize,
        destino: Vector2i,
    ) -> Option<(i32, i32)> {
        let celulas = self.jogador_humano.tabuleiro().obter_celulas_navio(navio_idx);
        if celulas.is_empty() {
            return None;
        }

        let mut candidatos = Vec::new();
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let mut contem_borda_nova = false;
            for &(x, y) in &celulas {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= BOARD_SIZE as i32 || ny >= BOARD_SIZE as i32 {
                    continue;
                }
                let in_old = celulas
                    .iter()
                    .any(|&(ox, oy)| ox == nx as usize && oy == ny as usize);
                if !in_old && destino == Vector2i::new(ny, nx) {
                    contem_borda_nova = true;
                    break;
                }
            }
            if contem_borda_nova {
                candidatos.push((dx, dy));
            }
        }

        if candidatos.len() == 1 {
            Some(candidatos[0])
        } else {
            None
        }
    }

    fn atualizar_preview_movimento_dinamico(&mut self) {
        let Some(mut preview_map) = self
            .base()
            .try_get_node_as::<TileMapLayer>("PreviewPosicionamento")
        else {
            return;
        };
        let Some(player_map) = self.base().try_get_node_as::<TileMapLayer>("CampoJogador") else {
            limpar_preview(&mut preview_map);
            return;
        };
        let Some(navio_idx) = self.navio_selecionado_movimento else {
            limpar_preview(&mut preview_map);
            return;
        };
        let mouse_pos = self.base().get_global_mouse_position();
        let Some((_, _, map_coord)) =
            conversao_coordenadas::clique_para_coordenada(player_map, mouse_pos)
        else {
            limpar_preview(&mut preview_map);
            return;
        };
        let Some((dx, dy)) = self.inferir_direcao_movimento_por_clique(navio_idx, map_coord) else {
            limpar_preview(&mut preview_map);
            return;
        };

        let celulas_atuais = self.jogador_humano.tabuleiro().obter_celulas_navio(navio_idx);
        let mut celulas_destino = Vec::new();
        for &(x, y) in &celulas_atuais {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && nx < BOARD_SIZE as i32 && ny < BOARD_SIZE as i32 {
                celulas_destino.push((nx as usize, ny as usize));
            }
        }

        let valido = self
            .jogador_humano
            .tabuleiro()
            .pode_mover_navio(navio_idx, dx, dy);
        let nome_navio = self
            .jogador_humano
            .tabuleiro()
            .navios
            .get(navio_idx)
            .map(|n| n.nome.as_str())
            .unwrap_or("Navio");

        render_preview_posicionamento(
            &mut preview_map,
            nome_navio,
            &celulas_destino,
            valido,
        );
    }

    fn executar_movimento_ia_dinamico(&mut self) {
        let Some(ref mut ia) = self.jogador_ia else {
            return;
        };
        let Some(movimento) = ia.escolher_movimento(&self.tiros_jogador_no_tabuleiro_ia) else {
            return;
        };

        let _ = ia
            .jogador_mut()
            .tabuleiro_mut()
            .mover_navio(movimento.navio_idx, movimento.dx, movimento.dy);
    }

    fn atualizar_controle_cursor(&mut self) {
        let mouse_pos = self.base().get_global_mouse_position();
        let estado = self.gerenciador_turnos.estado_atual();

        let (mostrar_jogador, mostrar_ia) = match estado {
            EstadoTurno::PosicionamentoJogador => (true, false),
            EstadoTurno::TurnoJogador => {
                if self.modo_dinamico && !self.movimento_jogador_realizado {
                    (true, true)
                } else {
                    (false, true)
                }
            }
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
        if let (Some(mut board_map), Some(mut ship_map)) = (
            self.base().try_get_node_as::<TileMapLayer>("CampoJogador"),
            self.mapa_navios_jogador.clone(),
        ) {
            render_tabuleiro_jogador(&mut board_map, &mut ship_map, self.jogador_humano.tabuleiro());
        }
    }
}
