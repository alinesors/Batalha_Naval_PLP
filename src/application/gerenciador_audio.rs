use godot::classes::{AudioStream, AudioStreamMp3, AudioStreamPlayer, ResourceLoader};
use godot::prelude::*;

use crate::domain::disparo::ResultadoDisparo;

const DELAY_SOM_RESULTADO: f64 = 0.4;

pub struct GerenciadorAudio {
    sfx_disparo: Option<Gd<AudioStreamPlayer>>,
    sfx_acerto: Option<Gd<AudioStreamPlayer>>,
    sfx_splash: Option<Gd<AudioStreamPlayer>>,
    sfx_destruicao: Option<Gd<AudioStreamPlayer>>,
    sfx_vitoria: Option<Gd<AudioStreamPlayer>>,
    sfx_derrota: Option<Gd<AudioStreamPlayer>>,
    musica_fundo_batalha: Option<Gd<AudioStreamPlayer>>,
    som_ondas: Option<Gd<AudioStreamPlayer>>,
    tempo_delay_som: f64,
    resultado_som_pendente: Option<ResultadoDisparo>,
}

impl GerenciadorAudio {
    pub fn novo() -> Self {
        Self {
            sfx_disparo: None,
            sfx_acerto: None,
            sfx_splash: None,
            sfx_destruicao: None,
            sfx_vitoria: None,
            sfx_derrota: None,
            musica_fundo_batalha: None,
            som_ondas: None,
            tempo_delay_som: 0.0,
            resultado_som_pendente: None,
        }
    }

    pub fn inicializar(&mut self, node: &Gd<Node>) {
        let mut resource_loader = ResourceLoader::singleton();

        // Carregar efeitos sonoros
        if let Some(mut sfx_disparo) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/sfx_disparo") {
            if let Some(resource) = resource_loader.load("res://sounds/disparo.mp3") {
                let stream = resource.cast::<AudioStream>();
                sfx_disparo.set_stream(&stream);
                sfx_disparo.set_volume_db(-5.0); // Volume moderado
            }
            self.sfx_disparo = Some(sfx_disparo);
        }

        if let Some(mut sfx_acerto) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/sfx_acerto") {
            if let Some(resource) = resource_loader.load("res://sounds/acerto.mp3") {
                let stream = resource.cast::<AudioStream>();
                sfx_acerto.set_stream(&stream);
                sfx_acerto.set_volume_db(-3.0); // Um pouco mais alto para feedback positivo
            }
            self.sfx_acerto = Some(sfx_acerto);
        }

        if let Some(mut sfx_splash) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/sfx_splash") {
            if let Some(resource) = resource_loader.load("res://sounds/splash.mp3") {
                let stream = resource.cast::<AudioStream>();
                sfx_splash.set_stream(&stream);
                sfx_splash.set_volume_db(-8.0); // Mais baixo para não cansar
            }
            self.sfx_splash = Some(sfx_splash);
        }

        if let Some(mut sfx_destruicao) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/sfx_destruicao") {
            if let Some(resource) = resource_loader.load("res://sounds/destruicao.mp3") {
                let stream = resource.cast::<AudioStream>();
                sfx_destruicao.set_stream(&stream);
                sfx_destruicao.set_volume_db(0.0); // Volume máximo para impacto
            }
            self.sfx_destruicao = Some(sfx_destruicao);
        }

        // Sons de fim de jogo
        if let Some(mut sfx_vitoria) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/sfx_som_vitoria") {
            if let Some(resource) = resource_loader.load("res://sounds/som_vitoria.mp3") {
                let stream = resource.cast::<AudioStream>();
                sfx_vitoria.set_stream(&stream);
                sfx_vitoria.set_volume_db(2.0); // Alto para celebração!
            }
            self.sfx_vitoria = Some(sfx_vitoria);
        }

        if let Some(mut sfx_derrota) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/sfx_som_derrota") {
            if let Some(resource) = resource_loader.load("res://sounds/som_derrota.mp3") {
                let stream = resource.cast::<AudioStream>();
                sfx_derrota.set_stream(&stream);
                sfx_derrota.set_volume_db(0.0); // Volume normal para derrota
            }
            self.sfx_derrota = Some(sfx_derrota);
        }

        // Carregar músicas de fundo
        if let Some(mut musica_batalha) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/musica_fundo_batalha") {
            if let Some(resource) = resource_loader.load("res://sounds/batalha.mp3") {
                let mut stream = resource.cast::<AudioStreamMp3>();
                stream.set_loop(true); // Loop infinito
                musica_batalha.set_stream(&stream.upcast::<AudioStream>());
                musica_batalha.set_volume_db(-6.0); // Música de fundo mais baixa
                musica_batalha.set_autoplay(false);
            }
            self.musica_fundo_batalha = Some(musica_batalha);
        }

        // Som ambiente de ondas (acelerado para ficar épico)
        if let Some(mut som_ondas) = node.try_get_node_as::<AudioStreamPlayer>("AudioManager/som_ondas") {
            if let Some(resource) = resource_loader.load("res://sounds/som_ondas.mp3") {
                let mut stream = resource.cast::<AudioStreamMp3>();
                stream.set_loop(true); // Loop infinito
                som_ondas.set_stream(&stream.upcast::<AudioStream>());
                som_ondas.set_volume_db(-22.0); // Muito baixo, apenas ambiente sutil
                som_ondas.set_pitch_scale(1.5); // Acelera 1.5x para ficar mais épico!
                som_ondas.set_autoplay(false);
            }
            self.som_ondas = Some(som_ondas);
        }
    }

    // Efeitos sonoros
    pub fn tocar_disparo(&mut self) {
        if let Some(sfx) = self.sfx_disparo.as_mut() {
            sfx.play();
        }
    }

    pub fn tocar_acerto(&mut self) {
        if let Some(sfx) = self.sfx_acerto.as_mut() {
            sfx.play();
        }
    }

    pub fn tocar_splash(&mut self) {
        if let Some(sfx) = self.sfx_splash.as_mut() {
            sfx.play();
        }
    }

    pub fn tocar_destruicao(&mut self) {
        if let Some(sfx) = self.sfx_destruicao.as_mut() {
            sfx.play();
        }
    }

    // Músicas de fundo
    pub fn tocar_musica_batalha(&mut self) {
        if let Some(musica) = self.musica_fundo_batalha.as_mut() {
            if !musica.is_playing() {
                // Começa aos 10 segundos (parte mais animada)
                musica.play_ex().from_position(10.0).done();
            }
        }
    }

    // Som ambiente
    pub fn tocar_ondas(&mut self) {
        if let Some(ondas) = self.som_ondas.as_mut() {
            if !ondas.is_playing() {
                ondas.play();
            }
        }
    }

    // Sons de fim de jogo
    pub fn tocar_vitoria(&mut self) {
        if let Some(sfx) = self.sfx_vitoria.as_mut() {
            sfx.play();
        }
    }

    pub fn tocar_derrota(&mut self) {
        if let Some(sfx) = self.sfx_derrota.as_mut() {
            sfx.play();
        }
    }

    // Sistema de delay para sons de resultado
    pub fn processar_delays(&mut self, delta: f64) {
        if self.tempo_delay_som > 0.0 {
            self.tempo_delay_som -= delta;
            if self.tempo_delay_som <= 0.0 {
                self.tocar_som_resultado_pendente();
            }
        }
    }

    pub fn tocar_disparo_com_resultado(&mut self, resultado: &ResultadoDisparo) {
        // Tocar som de disparo imediatamente
        self.tocar_disparo();
        
        // Agendar som de resultado para tocar após o delay
        self.resultado_som_pendente = Some(resultado.clone());
        self.tempo_delay_som = DELAY_SOM_RESULTADO;
    }

    fn tocar_som_resultado_pendente(&mut self) {
        if let Some(resultado) = self.resultado_som_pendente.take() {
            match &resultado {
                ResultadoDisparo::Acerto => self.tocar_acerto(),
                ResultadoDisparo::Afundou(_) => {
                    self.tocar_acerto();
                    self.tocar_destruicao();
                }
                ResultadoDisparo::Agua => self.tocar_splash(),
                _ => {}
            }
        }
    }
}
