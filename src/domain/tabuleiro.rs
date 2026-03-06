use godot::prelude::*;
use godot::classes::RandomNumberGenerator;

pub const BOARD_SIZE: usize = 10;

pub struct ConfigNavio {
    pub nome: &'static str,
    pub tamanho: usize,
    pub quantidade: usize,
}

pub const FROTA_PADRAO: [ConfigNavio; 4] = [
    ConfigNavio { nome: "Porta-Aviões", tamanho: 6, quantidade: 2 },
    ConfigNavio { nome: "Navio de Guerra", tamanho: 4, quantidade: 2 },
    ConfigNavio { nome: "Encouraçado", tamanho: 3, quantidade: 1 },
    ConfigNavio { nome: "Submarino", tamanho: 1, quantidade: 1 },
];

#[derive(Clone, Debug)]
pub struct Navio {
    pub nome: String,
    pub tamanho: usize,
    pub acertos: usize,
}

impl Navio {
    pub fn novo(nome: &str, tamanho: usize) -> Self {
        Self {
            nome: nome.to_string(),
            tamanho,
            acertos: 0,
        }
    }
    pub fn esta_afundado(&self) -> bool {
        self.acertos >= self.tamanho
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Celula {
    Vazio,
    Ocupado(usize),
    Agua,
    Atingido(usize),
}

pub struct EstadoTabuleiro {
    cells: [[Celula; BOARD_SIZE]; BOARD_SIZE],
    pub navios: Vec<Navio>,
}

impl EstadoTabuleiro {
    pub fn vazio() -> Self {
        Self {
            cells: [[Celula::Vazio; BOARD_SIZE]; BOARD_SIZE],
            navios: Vec::new(),
        }
    }

    pub fn valor_celula(&self, x: usize, y: usize) -> Option<Celula> {
        if x >= BOARD_SIZE || y >= BOARD_SIZE { return None; }
        Some(self.cells[x][y])
    }

    pub fn definir_celula(&mut self, x: usize, y: usize, valor: Celula) {
        if x < BOARD_SIZE && y < BOARD_SIZE {
            self.cells[x][y] = valor;
        }
    }

    pub fn posicionar_navio(&mut self, nome: &str, x: usize, y: usize, tamanho: usize, horizontal: bool) -> Result<(), String> {
        for i in 0..tamanho {
            let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
            if nx >= BOARD_SIZE || ny >= BOARD_SIZE { return Err("Fora do mapa".into()); }
            if self.cells[nx][ny] != Celula::Vazio { return Err("Posição ocupada".into()); }
        }

        let navio_idx = self.navios.len();
        self.navios.push(Navio::novo(nome, tamanho));

        for i in 0..tamanho {
            let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
            self.cells[nx][ny] = Celula::Ocupado(navio_idx);
        }
        Ok(())
    }

    pub fn preencher_aleatoriamente(&mut self) {
        let mut rng = RandomNumberGenerator::new_gd();
        rng.randomize();

        for config in FROTA_PADRAO.iter() {
            for _ in 0..config.quantidade {
                let mut posicionado = false;
                while !posicionado {
                    let x = rng.randi_range(0, (BOARD_SIZE - 1) as i32) as usize;
                    let y = rng.randi_range(0, (BOARD_SIZE - 1) as i32) as usize;
                    let horizontal = rng.randf() > 0.5;

                    if self.posicionar_navio(config.nome, x, y, config.tamanho, horizontal).is_ok() {
                        posicionado = true;
                    }
                }
            }
        }
    }
}