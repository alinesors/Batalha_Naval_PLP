use godot::global::Key;
use godot::prelude::*;
use crate::domain::jogador_ia::JogadorIA;

pub struct FaseSelecaoDificuldade;

impl FaseSelecaoDificuldade {
    pub fn nova() -> Self {
        Self
    }

    pub fn processar_tecla(&self, tecla: Key) -> Option<JogadorIA> {
        match tecla {
            Key::KEY_1 => {
                godot_print!("Dificuldade selecionada: Fácil");
                Some(JogadorIA::novo_facil())
            }
            Key::KEY_2 => {
                godot_print!("Dificuldade selecionada: Médio");
                Some(JogadorIA::novo_intermediario())
            }
            Key::KEY_3 => {
                godot_print!("Dificuldade selecionada: Difícil");
                Some(JogadorIA::novo_dificil())
            }
            _ => None,
        }
    }

    pub fn processar_selecao(&self, dificuldade: u8) -> Option<JogadorIA> {
        match dificuldade {
            0 => {
                godot_print!("Dificuldade selecionada: Fácil");
                Some(JogadorIA::novo_facil())
            }
            1 => {
                godot_print!("Dificuldade selecionada: Médio");
                Some(JogadorIA::novo_intermediario())
            }
            2 => {
                godot_print!("Dificuldade selecionada: Difícil");
                Some(JogadorIA::novo_dificil())
            }
            _ => None,
        }
    }
}
