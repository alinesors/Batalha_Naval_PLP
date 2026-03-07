use std::collections::HashMap;
use godot::prelude::*;

use crate::domain::disparo::{ResultadoDisparo, RetornoDisparo};
use crate::domain::estrategias_ia::EstrategiaIA;
use crate::domain::tabuleiro::{EstadoTabuleiro, BOARD_SIZE, FROTA_PADRAO};

#[derive(Debug, Clone, Copy, PartialEq)]
enum EstadoCelula {
    Desconhecido,
    Agua,
    Acerto,
}

pub struct EstrategiaDificil {
    navios_restantes: HashMap<usize, usize>,
    acertos_ativos: Vec<(usize, usize)>,
    mapa_conhecimento: [[EstadoCelula; BOARD_SIZE]; BOARD_SIZE],
    todos_acertos: Vec<(usize, usize)>,
}

impl EstrategiaDificil {
    pub fn nova() -> Self {
        let mut navios_restantes = HashMap::new();
        
        for config in FROTA_PADRAO.iter() {
            *navios_restantes.entry(config.tamanho).or_insert(0) += config.quantidade;
        }
        
        Self {
            navios_restantes,
            acertos_ativos: Vec::new(),
            mapa_conhecimento: [[EstadoCelula::Desconhecido; BOARD_SIZE]; BOARD_SIZE],
            todos_acertos: Vec::new(),
        }
    }

    fn calcular_mapa_probabilidades(&self) -> [[f32; BOARD_SIZE]; BOARD_SIZE] {
        let mut probabilidades = [[0.0; BOARD_SIZE]; BOARD_SIZE];

        for (&tamanho, &quantidade) in &self.navios_restantes {
            if quantidade == 0 {
                continue;
            }

            for x in 0..BOARD_SIZE {
                for y in 0..BOARD_SIZE {
                    if self.pode_colocar_horizontal(x, y, tamanho) {
                        for i in 0..tamanho {
                            probabilidades[x][y + i] += 1.0;
                        }
                    }
                    
                    if self.pode_colocar_vertical(x, y, tamanho) {
                        for i in 0..tamanho {
                            probabilidades[x + i][y] += 1.0;
                        }
                    }
                }
            }
        }

        if self.acertos_ativos.is_empty() {
            let menor_navio = self.navios_restantes
                .iter()
                .filter(|(_, &qtd)| qtd > 0)
                .map(|(&tamanho, _)| tamanho)
                .min()
                .unwrap_or(1);

            for x in 0..BOARD_SIZE {
                for y in 0..BOARD_SIZE {
                    if probabilidades[x][y] > 0.0 {
                        if (x + y) % menor_navio == 0 {
                            probabilidades[x][y] *= 1.5;
                        }
                        
                        if x < 2 || x >= BOARD_SIZE - 2 || y < 2 || y >= BOARD_SIZE - 2 {
                            probabilidades[x][y] *= 1.15;
                        }
                    }
                }
            }
        }

        if !self.acertos_ativos.is_empty() {
            if self.acertos_ativos.len() == 1 {
                let (ax, ay) = self.acertos_ativos[0];
                
                if ax > 0 && self.mapa_conhecimento[ax - 1][ay] == EstadoCelula::Desconhecido {
                    probabilidades[ax - 1][ay] *= 10.0;
                }
                if ax + 1 < BOARD_SIZE && self.mapa_conhecimento[ax + 1][ay] == EstadoCelula::Desconhecido {
                    probabilidades[ax + 1][ay] *= 10.0;
                }
                if ay > 0 && self.mapa_conhecimento[ax][ay - 1] == EstadoCelula::Desconhecido {
                    probabilidades[ax][ay - 1] *= 10.0;
                }
                if ay + 1 < BOARD_SIZE && self.mapa_conhecimento[ax][ay + 1] == EstadoCelula::Desconhecido {
                    probabilidades[ax][ay + 1] *= 10.0;
                }
            } else {
                self.aumentar_probabilidade_direcional(&mut probabilidades);
            }
        }

        let mut rng = godot::classes::RandomNumberGenerator::new_gd();
        rng.randomize();
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if probabilidades[x][y] > 0.0 {
                    let noise = rng.randf_range(0.90, 1.10);
                    probabilidades[x][y] *= noise;
                }
            }
        }

        probabilidades
    }

    fn pode_colocar_horizontal(&self, x: usize, y: usize, tamanho: usize) -> bool {
        if y + tamanho > BOARD_SIZE {
            return false;
        }

        let mut tem_acerto = false;
        for i in 0..tamanho {
            let estado = self.mapa_conhecimento[x][y + i];
            match estado {
                EstadoCelula::Desconhecido => {},
                EstadoCelula::Acerto => tem_acerto = true,
                EstadoCelula::Agua => return false,
            }
        }

        if !self.acertos_ativos.is_empty() {
            return tem_acerto;
        }

        true
    }

    fn pode_colocar_vertical(&self, x: usize, y: usize, tamanho: usize) -> bool {
        if x + tamanho > BOARD_SIZE {
            return false;
        }

        let mut tem_acerto = false;
        for i in 0..tamanho {
            let estado = self.mapa_conhecimento[x + i][y];
            match estado {
                EstadoCelula::Desconhecido => {},
                EstadoCelula::Acerto => tem_acerto = true,
                EstadoCelula::Agua => return false,
            }
        }

        if !self.acertos_ativos.is_empty() {
            return tem_acerto;
        }

        true
    }

    fn aumentar_probabilidade_direcional(&self, probabilidades: &mut [[f32; BOARD_SIZE]; BOARD_SIZE]) {
        if self.acertos_ativos.len() < 2 {
            return;
        }

        let mut acertos_ordenados = self.acertos_ativos.clone();
        acertos_ordenados.sort();

        let primeiro = acertos_ordenados[0];

        let todos_mesma_linha = acertos_ordenados.iter().all(|&(x, _)| x == primeiro.0);
        let todos_mesma_coluna = acertos_ordenados.iter().all(|&(_, y)| y == primeiro.1);

        if todos_mesma_linha {
            let x = primeiro.0;
            let y_min = acertos_ordenados.iter().map(|&(_, y)| y).min().unwrap();
            let y_max = acertos_ordenados.iter().map(|&(_, y)| y).max().unwrap();
            
            if y_min > 0 && self.mapa_conhecimento[x][y_min - 1] == EstadoCelula::Desconhecido {
                probabilidades[x][y_min - 1] *= 50.0;
            }
            if y_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y_max + 1] == EstadoCelula::Desconhecido {
                probabilidades[x][y_max + 1] *= 50.0;
            }
        } else if todos_mesma_coluna {
            let y = primeiro.1;
            let x_min = acertos_ordenados.iter().map(|&(x, _)| x).min().unwrap();
            let x_max = acertos_ordenados.iter().map(|&(x, _)| x).max().unwrap();
            
            if x_min > 0 && self.mapa_conhecimento[x_min - 1][y] == EstadoCelula::Desconhecido {
                probabilidades[x_min - 1][y] *= 50.0;
            }
            if x_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x_max + 1][y] == EstadoCelula::Desconhecido {
                probabilidades[x_max + 1][y] *= 50.0;
            }
        } else {
            for &(ax, ay) in &self.acertos_ativos {
                if ax > 0 && self.mapa_conhecimento[ax - 1][ay] == EstadoCelula::Desconhecido {
                    probabilidades[ax - 1][ay] *= 10.0;
                }
                if ax + 1 < BOARD_SIZE && self.mapa_conhecimento[ax + 1][ay] == EstadoCelula::Desconhecido {
                    probabilidades[ax + 1][ay] *= 10.0;
                }
                if ay > 0 && self.mapa_conhecimento[ax][ay - 1] == EstadoCelula::Desconhecido {
                    probabilidades[ax][ay - 1] *= 10.0;
                }
                if ay + 1 < BOARD_SIZE && self.mapa_conhecimento[ax][ay + 1] == EstadoCelula::Desconhecido {
                    probabilidades[ax][ay + 1] *= 10.0;
                }
            }
        }
    }

    fn escolher_melhor_celula(&self, probabilidades: &[[f32; BOARD_SIZE]; BOARD_SIZE]) -> Option<(usize, usize)> {
        let mut melhor_prob = -1.0;
        let mut candidatos = Vec::new();

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if self.mapa_conhecimento[x][y] != EstadoCelula::Desconhecido {
                    continue;
                }
                
                let prob = probabilidades[x][y];
                
                if prob > melhor_prob {
                    melhor_prob = prob;
                    candidatos.clear();
                    candidatos.push((x, y));
                } else if prob > 0.0 && (prob - melhor_prob).abs() < 0.001 {
                    candidatos.push((x, y));
                }
            }
        }

        if candidatos.is_empty() {
            return None;
        }

        let mut rng = godot::classes::RandomNumberGenerator::new_gd();
        rng.randomize();
        let idx = rng.randi_range(0, (candidatos.len() - 1) as i32) as usize;
        Some(candidatos[idx])
    }

    fn encontrar_sequencia_conectada(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut sequencia = vec![(x, y)];
        
        let mut idx_horizontal = 0;
        while idx_horizontal < sequencia.len() {
            let (cx, cy) = sequencia[idx_horizontal];
            
            if cy > 0 && self.mapa_conhecimento[cx][cy - 1] == EstadoCelula::Acerto {
                let pos = (cx, cy - 1);
                if !sequencia.contains(&pos) {
                    sequencia.push(pos);
                }
            }
            if cy + 1 < BOARD_SIZE && self.mapa_conhecimento[cx][cy + 1] == EstadoCelula::Acerto {
                let pos = (cx, cy + 1);
                if !sequencia.contains(&pos) {
                    sequencia.push(pos);
                }
            }
            
            idx_horizontal += 1;
        }
        
        let mut idx_vertical = 0;
        while idx_vertical < sequencia.len() {
            let (cx, cy) = sequencia[idx_vertical];
            
            if cx > 0 && self.mapa_conhecimento[cx - 1][cy] == EstadoCelula::Acerto {
                let pos = (cx - 1, cy);
                if !sequencia.contains(&pos) {
                    sequencia.push(pos);
                }
            }
            if cx + 1 < BOARD_SIZE && self.mapa_conhecimento[cx + 1][cy] == EstadoCelula::Acerto {
                let pos = (cx + 1, cy);
                if !sequencia.contains(&pos) {
                    sequencia.push(pos);
                }
            }
            
            idx_vertical += 1;
        }
        
        sequencia
    }
}

impl EstrategiaIA for EstrategiaDificil {
    fn escolher_alvo(&mut self, _tabuleiro: &EstadoTabuleiro) -> Option<(usize, usize)> {
        let probabilidades = self.calcular_mapa_probabilidades();
        
        self.escolher_melhor_celula(&probabilidades)
    }

    fn notificar_resultado(&mut self, x: usize, y: usize, resultado: &RetornoDisparo) {
        match &resultado.resultado {
            ResultadoDisparo::Agua => {
                self.mapa_conhecimento[x][y] = EstadoCelula::Agua;
                // A confirmação de navios destruídos agora vem exclusivamente do som de destruição
            }
            ResultadoDisparo::Acerto => {
                self.mapa_conhecimento[x][y] = EstadoCelula::Acerto;
                
                if !self.todos_acertos.contains(&(x, y)) {
                    self.todos_acertos.push((x, y));
                }
                
                if !self.acertos_ativos.contains(&(x, y)) {
                    self.acertos_ativos.push((x, y));
                }
            }
            ResultadoDisparo::Afundou(nome_navio) => {
                self.mapa_conhecimento[x][y] = EstadoCelula::Acerto;
                
                if !self.todos_acertos.contains(&(x, y)) {
                    self.todos_acertos.push((x, y));
                }

                // A IA "ouve" o som de destruição e tem certeza absoluta!
                let sequencia = self.encontrar_sequencia_conectada(x, y);
                let tamanho_calculado = sequencia.len();

                // Confia totalmente no som de destruição - o navio foi completamente destruído
                if let Some(count) = self.navios_restantes.get_mut(&tamanho_calculado) {
                    if *count > 0 {
                        *count -= 1;
                        godot_print!("IA Difícil: 🔊 Som de destruição detectado! {} confirmado afundado (tamanho {})! Restantes: {:?}", 
                            nome_navio, tamanho_calculado, self.navios_restantes);
                    }
                }

                // Remove todos os acertos dessa sequência da lista ativa
                for &pos in &sequencia {
                    self.acertos_ativos.retain(|&p| p != pos);
                }
            }
            _ => {}
        }
    }
}
