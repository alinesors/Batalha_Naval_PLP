use crate::domain::disparo::RetornoDisparo;
use crate::domain::estrategias_ia::{EstrategiaFacil, EstrategiaIA, EstrategiaIntermediaria, EstrategiaDificil};
use crate::domain::jogador::Jogador;
use crate::domain::tabuleiro::EstadoTabuleiro;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Dificuldade {
    Facil,
    Intermediario,
    Dificil,
}

pub struct JogadorIA {
    jogador: Jogador,
    estrategia: Box<dyn EstrategiaIA>,
}

impl JogadorIA {
    pub fn novo(dificuldade: Dificuldade) -> Self {
        let estrategia: Box<dyn EstrategiaIA> = match dificuldade {
            Dificuldade::Facil => Box::new(EstrategiaFacil),
            Dificuldade::Intermediario => Box::new(EstrategiaIntermediaria::nova()),
            Dificuldade::Dificil => Box::new(EstrategiaDificil::nova()),
        };

        Self {
            jogador: Jogador::novo_ia(),
            estrategia,
        }
    }

    pub fn novo_facil() -> Self {
        Self::novo(Dificuldade::Facil)
    }

    pub fn novo_intermediario() -> Self {
        Self::novo(Dificuldade::Intermediario)
    }

    #[allow(dead_code)]
    pub fn novo_dificil() -> Self {
        Self::novo(Dificuldade::Dificil)
    }

    pub fn jogador_mut(&mut self) -> &mut Jogador {
        &mut self.jogador
    }
    
    
    pub fn tabuleiro(&self) -> &EstadoTabuleiro {
        self.jogador.tabuleiro()
    }

    pub fn escolher_alvo(&mut self, tabuleiro_inimigo: &EstadoTabuleiro) -> Option<(usize, usize)> {
        self.estrategia.escolher_alvo(tabuleiro_inimigo)
    }

    pub fn notificar_resultado(&mut self, x: usize, y: usize, resultado: &RetornoDisparo) {
        self.estrategia.notificar_resultado(x, y, resultado);
    }

    pub fn receber_disparo(&mut self, x: usize, y: usize) -> RetornoDisparo {
        self.jogador.receber_disparo(x, y)
    }

    pub fn perdeu(&self) -> bool {
        self.jogador.perdeu()
    }
}
