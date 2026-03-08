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
    Vazio,          // Célula vazia (água sem tiro)
    AguaAtirada,    // Água com tiro (erro)
    Ocupado(usize), // Navio intacto
    Atingido(usize),// Navio parcialmente atingido
    Afundado(usize),// Navio completamente destruído
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

    /// Transforma todas as células Atingido(idx) em Afundado(idx) quando um navio afunda
    pub fn afundar_navio(&mut self, navio_idx: usize) {
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if let Celula::Atingido(idx) = self.cells[x][y] {
                    if idx == navio_idx {
                        self.cells[x][y] = Celula::Afundado(navio_idx);
                    }
                }
            }
        }
    }

    pub fn validar_posicao_navio(
        &self,
        x: usize,
        y: usize,
        tamanho: usize,
        horizontal: bool,
    ) -> Result<(), String> {
        for i in 0..tamanho {
            let (nx, ny) = if horizontal { (x, y + i) } else { (x + i, y) };
            if nx >= BOARD_SIZE || ny >= BOARD_SIZE {
                return Err("Fora do mapa".into());
            }
            if self.cells[nx][ny] != Celula::Vazio {
                return Err("Posição ocupada".into());
            }
        }
        Ok(())
    }

    pub fn pode_posicionar_navio(&self, x: usize, y: usize, tamanho: usize, horizontal: bool) -> bool {
        self.validar_posicao_navio(x, y, tamanho, horizontal).is_ok()
    }

    pub fn posicionar_navio(&mut self, nome: &str, x: usize, y: usize, tamanho: usize, horizontal: bool) -> Result<(), String> {
        self.validar_posicao_navio(x, y, tamanho, horizontal)?;

        let navio_idx = self.navios.len();
        self.navios.push(Navio::novo(nome, tamanho));

        for i in 0..tamanho {
            let (nx, ny) = if horizontal { (x, y + i) } else { (x + i, y) };
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

    pub fn remover_navio_na_posicao(&mut self, x: usize, y: usize) -> Option<String> {
        // Verificar se há um navio nesta posição
        let navio_idx = match self.cells[x][y] {
            Celula::Ocupado(idx) | Celula::Atingido(idx) | Celula::Afundado(idx) => Some(idx),
            _ => None,
        }?;
        
        let nome_navio = self.navios.get(navio_idx)?.nome.clone();
        
        // Limpar células que contêm esse navio
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                match self.cells[x][y] {
                    Celula::Ocupado(navio_id) | Celula::Atingido(navio_id) | Celula::Afundado(navio_id) 
                        if navio_id == navio_idx => {
                        self.cells[x][y] = Celula::Vazio;
                    }
                    _ => {}
                }
            }
        }

        // Remover navio da lista
        self.navios.remove(navio_idx);

        // Atualizar índices nas células (todos os navios depois desse índice devem ser decrementados)
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                match self.cells[x][y] {
                    Celula::Ocupado(navio_id) if navio_id > navio_idx => {
                        self.cells[x][y] = Celula::Ocupado(navio_id - 1);
                    }
                    Celula::Atingido(navio_id) if navio_id > navio_idx => {
                        self.cells[x][y] = Celula::Atingido(navio_id - 1);
                    }
                    Celula::Afundado(navio_id) if navio_id > navio_idx => {
                        self.cells[x][y] = Celula::Afundado(navio_id - 1);
                    }
                    _ => {}
                }
            }
        }

        Some(nome_navio)
    }
}
