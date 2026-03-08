use godot::classes::{Button, FontFile, Label, Node2D, ResourceLoader};
use godot::global::HorizontalAlignment;
use godot::prelude::*;

use crate::application::gerenciador_turnos::EstadoTurno;

pub struct GerenciadorInterface {
    label_fase: Option<Gd<Label>>,
    label_turno: Option<Gd<Label>>,
    label_resultado: Option<Gd<Label>>,
    label_reiniciar: Option<Gd<Label>>,
    label_instrucao_dificuldade: Option<Gd<Label>>,
    label_ajuda_posicionamento: Option<Gd<Label>>,
    botao_confirmar_posicionamento: Option<Gd<Button>>,
}

impl GerenciadorInterface {
    pub fn novo() -> Self {
        Self {
            label_fase: None,
            label_turno: None,
            label_resultado: None,
            label_reiniciar: None,
            label_instrucao_dificuldade: None,
            label_ajuda_posicionamento: None,
            botao_confirmar_posicionamento: None,
        }
    }

    pub fn inicializar(&mut self, node: Gd<Node2D>) {
        let mut resource_loader = ResourceLoader::singleton();
        let font_path = "res://fonts/Retro Gaming.ttf";
        
        let font = resource_loader
            .load(font_path)
            .and_then(|res| res.try_cast::<FontFile>().ok());

        if let Some(mut label_fase) = node.try_get_node_as::<Label>("LabelFase") {
            if let Some(font_file) = font.clone() {
                label_fase.add_theme_font_override("font", &font_file);
            }
            label_fase.add_theme_font_size_override("font_size", 24);
            label_fase.add_theme_color_override("font_color", Color::from_rgb(0.0, 0.0, 0.0));
            label_fase.add_theme_color_override("font_outline_color", Color::from_rgb(1.0, 1.0, 1.0));
            label_fase.add_theme_constant_override("outline_size", 3);
            label_fase.set_horizontal_alignment(HorizontalAlignment::CENTER);
            label_fase.set_position(Vector2::new(100.0, 20.0));
            label_fase.set_size(Vector2::new(400.0, 50.0));
            label_fase.set_z_index(100);
            self.label_fase = Some(label_fase);
        }

        if let Some(mut label_turno) = node.try_get_node_as::<Label>("LabelTurno") {
            if let Some(font_file) = font.clone() {
                label_turno.add_theme_font_override("font", &font_file);
            }
            label_turno.add_theme_font_size_override("font_size", 18);
            label_turno.add_theme_color_override("font_color", Color::from_rgb(0.0, 0.0, 0.0));
            label_turno.add_theme_color_override("font_outline_color", Color::from_rgb(1.0, 1.0, 1.0));
            label_turno.add_theme_constant_override("outline_size", 3);
            label_turno.set_horizontal_alignment(HorizontalAlignment::CENTER);
            label_turno.set_position(Vector2::new(100.0, 70.0));
            label_turno.set_size(Vector2::new(400.0, 40.0));
            label_turno.set_z_index(100);
            self.label_turno = Some(label_turno);
        }

        if let Some(mut label_resultado) = node.try_get_node_as::<Label>("LabelResultado") {
            if let Some(ref font_file) = font {
                label_resultado.add_theme_font_override("font", font_file);
            }
            label_resultado.add_theme_font_size_override("font_size", 32);
            label_resultado.add_theme_color_override("font_color", Color::from_rgb(0.0, 0.0, 0.0));
            label_resultado.add_theme_color_override("font_outline_color", Color::from_rgb(1.0, 1.0, 1.0));
            label_resultado.add_theme_constant_override("outline_size", 4);
            label_resultado.set_horizontal_alignment(HorizontalAlignment::CENTER);
            label_resultado.set_position(Vector2::new(100.0, 300.0));
            label_resultado.set_size(Vector2::new(400.0, 60.0));
            label_resultado.set_z_index(100);
            label_resultado.set_visible(false);
            self.label_resultado = Some(label_resultado);
        }

        if let Some(mut label_reiniciar) = node.try_get_node_as::<Label>("LabelReiniciar") {
            if let Some(ref font_file) = font {
                label_reiniciar.add_theme_font_override("font", font_file);
            }
            label_reiniciar.add_theme_font_size_override("font_size", 18);
            label_reiniciar.add_theme_color_override("font_color", Color::from_rgb(0.0, 0.0, 0.0));
            label_reiniciar.add_theme_color_override("font_outline_color", Color::from_rgb(1.0, 1.0, 1.0));
            label_reiniciar.add_theme_constant_override("outline_size", 3);
            label_reiniciar.set_horizontal_alignment(HorizontalAlignment::CENTER);
            label_reiniciar.set_position(Vector2::new(100.0, 360.0));
            label_reiniciar.set_size(Vector2::new(400.0, 40.0));
            label_reiniciar.set_z_index(100);
            label_reiniciar.set_text("Pressione R para reiniciar");
            label_reiniciar.set_visible(false);
            self.label_reiniciar = Some(label_reiniciar);
        }

        if let Some(mut label_instrucao) = node.try_get_node_as::<Label>("LabelInstrucaoDificuldade") {
            if let Some(ref font_file) = font {
                label_instrucao.add_theme_font_override("font", font_file);
            }
            label_instrucao.add_theme_font_size_override("font_size", 28);
            label_instrucao.add_theme_color_override("font_color", Color::from_rgb(0.0, 0.0, 0.0));
            label_instrucao.add_theme_color_override("font_outline_color", Color::from_rgb(1.0, 1.0, 1.0));
            label_instrucao.add_theme_constant_override("outline_size", 4);
            label_instrucao.set_horizontal_alignment(HorizontalAlignment::CENTER);
            label_instrucao.set_position(Vector2::new(100.0, 300.0));
            label_instrucao.set_size(Vector2::new(400.0, 50.0));
            label_instrucao.set_z_index(100);
            label_instrucao.set_text("Selecione a Dificuldade");
            label_instrucao.set_visible(false);
            self.label_instrucao_dificuldade = Some(label_instrucao);
        }

        if let Some(mut label_ajuda) = node.try_get_node_as::<Label>("LabelAjudaPosicionamento") {
            if let Some(ref font_file) = font {
                label_ajuda.add_theme_font_override("font", font_file);
            }
            label_ajuda.add_theme_font_size_override("font_size", 16);
            label_ajuda.add_theme_color_override("font_color", Color::from_rgb(0.0, 0.0, 0.0));
            label_ajuda.add_theme_color_override("font_outline_color", Color::from_rgb(1.0, 1.0, 1.0));
            label_ajuda.add_theme_constant_override("outline_size", 3);
            label_ajuda.set_horizontal_alignment(HorizontalAlignment::CENTER);
            label_ajuda.set_position(Vector2::new(100.0, 50.0));
            label_ajuda.set_size(Vector2::new(400.0, 30.0));
            label_ajuda.set_z_index(100);
            label_ajuda.set_text("Aperte R para girar os navios");
            label_ajuda.set_visible(false);
            self.label_ajuda_posicionamento = Some(label_ajuda);
        }

        if let Some(mut botao) = node.try_get_node_as::<Button>("BotaoConfirmarPosicionamento") {
            if let Some(font_file) = font.clone() {
                botao.add_theme_font_override("font", &font_file);
            }
            botao.add_theme_font_size_override("font_size", 22);
            botao.set_position(Vector2::new(180.0, 320.0));
            botao.set_size(Vector2::new(240.0, 50.0));
            botao.set_text("Começar Batalha");
            botao.set_visible(false);
            self.botao_confirmar_posicionamento = Some(botao);
        }
    }

    pub fn atualizar(&mut self, estado: EstadoTurno, rodada: u32) {
        match estado {
            EstadoTurno::SelecaoDificuldade => {
                if let Some(mut label_fase) = self.label_fase.clone() {
                    label_fase.set_visible(false);
                }
                if let Some(mut label_turno) = self.label_turno.clone() {
                    label_turno.set_visible(false);
                }
                if let Some(mut label_resultado) = self.label_resultado.clone() {
                    label_resultado.set_visible(false);
                }
                if let Some(mut label_reiniciar) = self.label_reiniciar.clone() {
                    label_reiniciar.set_visible(false);
                }
                if let Some(mut label_instrucao) = self.label_instrucao_dificuldade.clone() {
                    label_instrucao.set_visible(true);
                }
                if let Some(mut label_ajuda) = self.label_ajuda_posicionamento.clone() {
                    label_ajuda.set_visible(false);
                }
            }
            EstadoTurno::PosicionamentoJogador => {
                if let Some(mut label_fase) = self.label_fase.clone() {
                    label_fase.set_text("Fase de Posicionamento");
                    label_fase.set_visible(true);
                }
                if let Some(mut label_turno) = self.label_turno.clone() {
                    label_turno.set_visible(false);
                }
                if let Some(mut label_resultado) = self.label_resultado.clone() {
                    label_resultado.set_visible(false);
                }
                if let Some(mut label_reiniciar) = self.label_reiniciar.clone() {
                    label_reiniciar.set_visible(false);
                }
                if let Some(mut label_instrucao) = self.label_instrucao_dificuldade.clone() {
                    label_instrucao.set_visible(false);
                }
                if let Some(mut label_ajuda) = self.label_ajuda_posicionamento.clone() {
                    label_ajuda.set_visible(true);
                }
            }
            EstadoTurno::TurnoJogador | EstadoTurno::TurnoIA => {
                if let Some(mut label_fase) = self.label_fase.clone() {
                    label_fase.set_text(&format!("Rodada {}", rodada));
                    label_fase.set_visible(true);
                }
                if let Some(mut label_turno) = self.label_turno.clone() {
                    if estado == EstadoTurno::TurnoJogador {
                        label_turno.set_text("Sua vez!");
                    } else {
                        label_turno.set_text("Turno da IA");
                    }
                    label_turno.set_visible(true);
                }
                if let Some(mut label_resultado) = self.label_resultado.clone() {
                    label_resultado.set_visible(false);
                }
                if let Some(mut label_reiniciar) = self.label_reiniciar.clone() {
                    label_reiniciar.set_visible(false);
                }
                if let Some(mut label_ajuda) = self.label_ajuda_posicionamento.clone() {
                    label_ajuda.set_visible(false);
                }
            }
            EstadoTurno::VitoriaJogador => {
                if let Some(mut label_fase) = self.label_fase.clone() {
                    label_fase.set_visible(false);
                }
                if let Some(mut label_turno) = self.label_turno.clone() {
                    label_turno.set_visible(false);
                }
                if let Some(mut label_resultado) = self.label_resultado.clone() {
                    label_resultado.set_text("Vitoria!");
                    label_resultado.set_visible(true);
                }
                if let Some(mut label_reiniciar) = self.label_reiniciar.clone() {
                    label_reiniciar.set_visible(true);
                }
                if let Some(mut label_ajuda) = self.label_ajuda_posicionamento.clone() {
                    label_ajuda.set_visible(false);
                }
            }
            EstadoTurno::VitoriaIA => {
                if let Some(mut label_fase) = self.label_fase.clone() {
                    label_fase.set_visible(false);
                }
                if let Some(mut label_turno) = self.label_turno.clone() {
                    label_turno.set_visible(false);
                }
                if let Some(mut label_resultado) = self.label_resultado.clone() {
                    label_resultado.set_text("Derrota!");
                    label_resultado.set_visible(true);
                }
                if let Some(mut label_reiniciar) = self.label_reiniciar.clone() {
                    label_reiniciar.set_visible(true);
                }
                if let Some(mut label_ajuda) = self.label_ajuda_posicionamento.clone() {
                    label_ajuda.set_visible(false);
                }
            }
            EstadoTurno::PosicionamentoIA => {
                // Estado transitório, não precisa mostrar nada
            }
        }
    }

    pub fn mostrar_botao_confirmar(&mut self) {
        if let Some(mut botao) = self.botao_confirmar_posicionamento.clone() {
            botao.set_visible(true);
        }
    }

    pub fn esconder_botao_confirmar(&mut self) {
        if let Some(mut botao) = self.botao_confirmar_posicionamento.clone() {
            botao.set_visible(false);
        }
    }
}
