use crate::domain::tabuleiro::{EstadoTabuleiro, Celula};

#[derive(Debug, Clone, PartialEq)]
pub enum ResultadoDisparo {
    Agua,
    Acerto,
    Afundou(String),
    JaDisparado,
    ForaDosLimites,
}

pub struct RetornoDisparo {
    pub resultado: ResultadoDisparo,
    pub mensagem: String,
}

pub fn executar_disparo(tabuleiro: &mut EstadoTabuleiro, x: usize, y: usize) -> RetornoDisparo {
    let Some(celula_atual) = tabuleiro.valor_celula(x, y) else {
        return RetornoDisparo {
            resultado: ResultadoDisparo::ForaDosLimites,
            mensagem: "Alvo fora do mapa!".to_string(),
        };
    };

    match celula_atual {
        Celula::Vazio => {
            tabuleiro.definir_celula(x, y, Celula::Agua);
            RetornoDisparo {
                resultado: ResultadoDisparo::Agua,
                mensagem: format!("Água em [{}, {}]!", x, y),
            }
        }
        Celula::Ocupado(idx) => {
            tabuleiro.definir_celula(x, y, Celula::Atingido(idx));
            let navio = &mut tabuleiro.navios[idx];
            navio.acertos += 1;

            if navio.esta_afundado() {
                let nome = navio.nome.clone();
                RetornoDisparo {
                    resultado: ResultadoDisparo::Afundou(nome.clone()),
                    mensagem: format!("KABOOM! O {} afundou!", nome),
                }
            } else {
                RetornoDisparo {
                    resultado: ResultadoDisparo::Acerto,
                    mensagem: format!("Fogo em [{}, {}]!", x, y),
                }
            }
        }
        Celula::Agua | Celula::Atingido(_) => {
            RetornoDisparo {
                resultado: ResultadoDisparo::JaDisparado,
                mensagem: "Você já atirou aqui!".to_string(),
            }
        }
    }
}