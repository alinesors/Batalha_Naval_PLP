#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoTurno {
    SelecaoDificuldade,
    PosicionamentoJogador,
    PosicionamentoIA,
    TurnoJogador,
    TurnoIA,
    VitoriaJogador,
    VitoriaIA,
}

pub struct GerenciadorTurnos {
    estado_atual: EstadoTurno,
    numero_turno: u32,
    navios_jogador: u32,
    navios_ia: u32,
}

impl GerenciadorTurnos {
    pub fn novo(total_navios_por_jogador: u32) -> Self {
        Self {
            estado_atual: EstadoTurno::SelecaoDificuldade,
            numero_turno: 0,
            navios_jogador: total_navios_por_jogador,
            navios_ia: total_navios_por_jogador,
        }
    }

    pub fn estado_atual(&self) -> EstadoTurno {
        self.estado_atual
    }

    pub fn confirmar_dificuldade(&mut self) {
        if self.estado_atual == EstadoTurno::SelecaoDificuldade {
            self.estado_atual = EstadoTurno::PosicionamentoJogador;
        }
    }

    pub fn finalizar_posicionamento_jogador(&mut self) {
        if self.estado_atual == EstadoTurno::PosicionamentoJogador {
            self.estado_atual = EstadoTurno::PosicionamentoIA;
        }
    }

    pub fn iniciar_jogo(&mut self) {
        if self.estado_atual == EstadoTurno::PosicionamentoIA {
            self.numero_turno = 1;
            self.estado_atual = EstadoTurno::TurnoJogador;
        }
    }

    pub fn processar_ataque_jogador(&mut self, _acertou: bool, afundou_navio: bool) -> bool {
        if self.estado_atual != EstadoTurno::TurnoJogador {
            return false;
        }

        if afundou_navio {
            self.navios_ia -= 1;
            if self.navios_ia == 0 {
                self.estado_atual = EstadoTurno::VitoriaJogador;
                return true;
            }
        }

        if _acertou {
            return true;
        }

        self.avancar_para_turno_ia();
        true
    }

    pub fn processar_ataque_ia(&mut self, _acertou: bool, afundou_navio: bool) -> bool {
        if self.estado_atual != EstadoTurno::TurnoIA {
            return false;
        }

        if afundou_navio {
            self.navios_jogador -= 1;
            if self.navios_jogador == 0 {
                self.estado_atual = EstadoTurno::VitoriaIA;
                return true;
            }
        }

        if _acertou {
            return true;
        }

        self.avancar_para_turno_jogador();
        true
    }

    pub fn jogo_terminou(&self) -> bool {
        matches!(
            self.estado_atual,
            EstadoTurno::VitoriaJogador | EstadoTurno::VitoriaIA
        )
    }

    pub fn rodada_atual(&self) -> u32 {
        self.numero_turno
    }

    pub fn forcar_vitoria_jogador(&mut self) {
        self.estado_atual = EstadoTurno::VitoriaJogador;
    }

    pub fn forcar_vitoria_ia(&mut self) {
        self.estado_atual = EstadoTurno::VitoriaIA;
    }

    fn avancar_para_turno_ia(&mut self) {
        self.estado_atual = EstadoTurno::TurnoIA;
    }

    fn avancar_para_turno_jogador(&mut self) {
        self.numero_turno += 1;
        self.estado_atual = EstadoTurno::TurnoJogador;
    }
}
