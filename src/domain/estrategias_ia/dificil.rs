use std::collections::HashMap;
use godot::prelude::*;

use crate::domain::disparo::{ResultadoDisparo, RetornoDisparo};
use crate::domain::estrategias_ia::EstrategiaIA;
use crate::domain::tabuleiro::{EstadoTabuleiro, BOARD_SIZE, FROTA_PADRAO};

#[derive(Debug, Clone, Copy, PartialEq)]
enum EstadoCelula {
    Desconhecido,
    Agua,
    Acerto,      // Navio atingido (ainda vivo)
    Afundado,    // Navio completamente destruído
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
                EstadoCelula::Agua | EstadoCelula::Afundado => return false,
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
                EstadoCelula::Agua | EstadoCelula::Afundado => return false,
            }
        }

        if !self.acertos_ativos.is_empty() {
            return tem_acerto;
        }

        true
    }

    /// Verifica se um grupo tem pontas disponíveis para atacar
    fn grupo_tem_pontas_disponiveis(&self, grupo: &[(usize, usize)]) -> bool {
        if grupo.len() < 2 {
            // Grupos de 1 célula sempre têm 4 direções possíveis
            let (x, y) = grupo[0];
            return (x > 0 && self.mapa_conhecimento[x - 1][y] == EstadoCelula::Desconhecido)
                || (x + 1 < BOARD_SIZE && self.mapa_conhecimento[x + 1][y] == EstadoCelula::Desconhecido)
                || (y > 0 && self.mapa_conhecimento[x][y - 1] == EstadoCelula::Desconhecido)
                || (y + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y + 1] == EstadoCelula::Desconhecido);
        }
        
        let mut ordenado = grupo.to_vec();
        ordenado.sort();
        
        let primeiro = ordenado[0];
        let todos_mesma_linha = ordenado.iter().all(|&(x, _)| x == primeiro.0);
        let todos_mesma_coluna = ordenado.iter().all(|&(_, y)| y == primeiro.1);
        
        if todos_mesma_linha {
            let x = primeiro.0;
            let y_min = ordenado.iter().map(|&(_, y)| y).min().unwrap();
            let y_max = ordenado.iter().map(|&(_, y)| y).max().unwrap();
            
            (y_min > 0 && self.mapa_conhecimento[x][y_min - 1] == EstadoCelula::Desconhecido)
                || (y_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y_max + 1] == EstadoCelula::Desconhecido)
        } else if todos_mesma_coluna {
            let y = primeiro.1;
            let x_min = ordenado.iter().map(|&(x, _)| x).min().unwrap();
            let x_max = ordenado.iter().map(|&(x, _)| x).max().unwrap();
            
            (x_min > 0 && self.mapa_conhecimento[x_min - 1][y] == EstadoCelula::Desconhecido)
                || (x_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x_max + 1][y] == EstadoCelula::Desconhecido)
        } else {
            // Grupo não linear (não deveria acontecer)
            false
        }
    }
    
    /// Retorna as pontas disponíveis de um grupo linear
    fn obter_pontas_disponiveis(&self, grupo: &[(usize, usize)]) -> Vec<(usize, usize)> {
        let mut pontas = Vec::new();
        
        if grupo.len() < 2 {
            // Acerto isolado - retornar todas as 4 direções disponíveis
            let (x, y) = grupo[0];
            if x > 0 && self.mapa_conhecimento[x - 1][y] == EstadoCelula::Desconhecido {
                pontas.push((x - 1, y));
            }
            if x + 1 < BOARD_SIZE && self.mapa_conhecimento[x + 1][y] == EstadoCelula::Desconhecido {
                pontas.push((x + 1, y));
            }
            if y > 0 && self.mapa_conhecimento[x][y - 1] == EstadoCelula::Desconhecido {
                pontas.push((x, y - 1));
            }
            if y + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y + 1] == EstadoCelula::Desconhecido {
                pontas.push((x, y + 1));
            }
            return pontas;
        }
        
        let mut ordenado = grupo.to_vec();
        ordenado.sort();
        
        let primeiro = ordenado[0];
        let todos_mesma_linha = ordenado.iter().all(|&(x, _)| x == primeiro.0);
        let todos_mesma_coluna = ordenado.iter().all(|&(_, y)| y == primeiro.1);
        
        if todos_mesma_linha {
            let x = primeiro.0;
            let y_min = ordenado.iter().map(|&(_, y)| y).min().unwrap();
            let y_max = ordenado.iter().map(|&(_, y)| y).max().unwrap();
            
            if y_min > 0 && self.mapa_conhecimento[x][y_min - 1] == EstadoCelula::Desconhecido {
                pontas.push((x, y_min - 1));
            }
            if y_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y_max + 1] == EstadoCelula::Desconhecido {
                pontas.push((x, y_max + 1));
            }
        } else if todos_mesma_coluna {
            let y = primeiro.1;
            let x_min = ordenado.iter().map(|&(x, _)| x).min().unwrap();
            let x_max = ordenado.iter().map(|&(x, _)| x).max().unwrap();
            
            if x_min > 0 && self.mapa_conhecimento[x_min - 1][y] == EstadoCelula::Desconhecido {
                pontas.push((x_min - 1, y));
            }
            if x_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x_max + 1][y] == EstadoCelula::Desconhecido {
                pontas.push((x_max + 1, y));
            }
        }
        
        pontas
    }

    /// Agrupa acertos ativos em sequências lineares separadas
    fn encontrar_grupos_lineares(&self) -> Vec<Vec<(usize, usize)>> {
        let mut grupos = Vec::new();
        let mut processados = Vec::new();
        
        for &acerto in &self.acertos_ativos {
            if processados.contains(&acerto) {
                continue;
            }
            
            // Encontra a sequência linear deste acerto
            let sequencia = self.encontrar_sequencia_linear_de_acertos(acerto.0, acerto.1);
            
            for &pos in &sequencia {
                processados.push(pos);
            }
            
            grupos.push(sequencia);
        }
        
        grupos
    }
    
    /// Similar a encontrar_sequencia_conectada, mas busca apenas em Acertos (não Afundados)
    fn encontrar_sequencia_linear_de_acertos(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut sequencia = vec![(x, y)];
        
        // Verificar adjacentes
        let esquerda = y > 0 && self.mapa_conhecimento[x][y - 1] == EstadoCelula::Acerto;
        let direita = y + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y + 1] == EstadoCelula::Acerto;
        let cima = x > 0 && self.mapa_conhecimento[x - 1][y] == EstadoCelula::Acerto;
        let baixo = x + 1 < BOARD_SIZE && self.mapa_conhecimento[x + 1][y] == EstadoCelula::Acerto;
        
        let tem_horizontal = esquerda || direita;
        let tem_vertical = cima || baixo;
        
        // Determinar direção (prioriza a mais longa em caso de intersecção)
        let direcao_horizontal = if tem_horizontal && tem_vertical {
            let mut count_h = 1;
            let mut cy = y;
            while cy > 0 && self.mapa_conhecimento[x][cy - 1] == EstadoCelula::Acerto {
                count_h += 1;
                cy -= 1;
            }
            cy = y;
            while cy + 1 < BOARD_SIZE && self.mapa_conhecimento[x][cy + 1] == EstadoCelula::Acerto {
                count_h += 1;
                cy += 1;
            }
            
            let mut count_v = 1;
            let mut cx = x;
            while cx > 0 && self.mapa_conhecimento[cx - 1][y] == EstadoCelula::Acerto {
                count_v += 1;
                cx -= 1;
            }
            cx = x;
            while cx + 1 < BOARD_SIZE && self.mapa_conhecimento[cx + 1][y] == EstadoCelula::Acerto {
                count_v += 1;
                cx += 1;
            }
            
            count_h >= count_v
        } else {
            tem_horizontal
        };
        
        if direcao_horizontal {
            // Expandir horizontalmente
            let mut cy = y;
            while cy > 0 && self.mapa_conhecimento[x][cy - 1] == EstadoCelula::Acerto {
                cy -= 1;
                sequencia.push((x, cy));
            }
            
            cy = y;
            while cy + 1 < BOARD_SIZE && self.mapa_conhecimento[x][cy + 1] == EstadoCelula::Acerto {
                cy += 1;
                sequencia.push((x, cy));
            }
        } else if tem_vertical {
            // Expandir verticalmente
            let mut cx = x;
            while cx > 0 && self.mapa_conhecimento[cx - 1][y] == EstadoCelula::Acerto {
                cx -= 1;
                sequencia.push((cx, y));
            }
            
            cx = x;
            while cx + 1 < BOARD_SIZE && self.mapa_conhecimento[cx + 1][y] == EstadoCelula::Acerto {
                cx += 1;
                sequencia.push((cx, y));
            }
        }
        
        sequencia
    }

    fn aumentar_probabilidade_direcional(&self, probabilidades: &mut [[f32; BOARD_SIZE]; BOARD_SIZE]) {
        if self.acertos_ativos.is_empty() {
            return;
        }
        
        // 🎯 Estratégia inteligente: agrupar acertos em sequências lineares separadas
        let grupos = self.encontrar_grupos_lineares();
        
        godot_print!("IA Difícil: 🧠 {} grupos de acertos detectados: {:?}", grupos.len(), grupos);
        
        // 🎯 ESTRATÉGIA INTELIGENTE DE PRIORIZAÇÃO:
        // 1. Grupos com 2+ células são SEMPRE prioritários (navio linear confirmado)
        // 2. Entre grupos 2+, focar no MAIOR (mais células = mais chances de acerto)
        // 3. Grupos de 1 célula só são considerados se não há grupos maiores
        
        let grupos_multicelula: Vec<_> = grupos.iter()
            .filter(|g| g.len() >= 2)
            .collect();
        
        let grupo_prioritario = if !grupos_multicelula.is_empty() {
            // Há navios lineares confirmados - focar no maior
            let grupo = grupos_multicelula.iter()
                .max_by_key(|g| g.len())
                .unwrap();
            godot_print!("IA Difícil: 🎯 NAVIO LINEAR detectado! Focando no grupo: {:?} (tamanho {})", grupo, grupo.len());
            *grupo
        } else {
            // Só acertos isolados - pegar qualquer um (maior primeiro)
            let grupo = grupos.iter()
                .max_by_key(|g| g.len())
                .unwrap();
            godot_print!("IA Difícil: 🎯 Apenas acertos isolados. Explorando: {:?}", grupo);
            grupo
        };
        
        // Se é apenas um acerto isolado, aumentar probabilidade ao redor (4 direções)
        if grupo_prioritario.len() == 1 {
            let (ax, ay) = grupo_prioritario[0];
            
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
            return;
        }
        
        // Múltiplos acertos - identificar se é linha ou coluna
        let mut grupo_ordenado = grupo_prioritario.clone();
        grupo_ordenado.sort();
        
        let primeiro = grupo_ordenado[0];
        let todos_mesma_linha = grupo_ordenado.iter().all(|&(x, _)| x == primeiro.0);
        let todos_mesma_coluna = grupo_ordenado.iter().all(|&(_, y)| y == primeiro.1);
        
        if todos_mesma_linha {
            // Navio horizontal - focar nas PONTAS (esquerda e direita)
            let x = primeiro.0;
            let y_min = grupo_ordenado.iter().map(|&(_, y)| y).min().unwrap();
            let y_max = grupo_ordenado.iter().map(|&(_, y)| y).max().unwrap();
            
            godot_print!("IA Difícil: ➡️ Navio HORIZONTAL detectado em linha {}, colunas {}-{}", x, y_min, y_max);
            
            if y_min > 0 && self.mapa_conhecimento[x][y_min - 1] == EstadoCelula::Desconhecido {
                probabilidades[x][y_min - 1] *= 100.0;
                godot_print!("IA Difícil: ⬅️ Prioridade MÁXIMA em ({}, {})", x, y_min - 1);
            }
            if y_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y_max + 1] == EstadoCelula::Desconhecido {
                probabilidades[x][y_max + 1] *= 100.0;
                godot_print!("IA Difícil: ➡️ Prioridade MÁXIMA em ({}, {})", x, y_max + 1);
            }
        } else if todos_mesma_coluna {
            // Navio vertical - focar nas PONTAS (cima e baixo)
            let y = primeiro.1;
            let x_min = grupo_ordenado.iter().map(|&(x, _)| x).min().unwrap();
            let x_max = grupo_ordenado.iter().map(|&(x, _)| x).max().unwrap();
            
            godot_print!("IA Difícil: ⬇️ Navio VERTICAL detectado em coluna {}, linhas {}-{}", y, x_min, x_max);
            
            if x_min > 0 && self.mapa_conhecimento[x_min - 1][y] == EstadoCelula::Desconhecido {
                probabilidades[x_min - 1][y] *= 100.0;
                godot_print!("IA Difícil: ⬆️ Prioridade MÁXIMA em ({}, {})", x_min - 1, y);
            }
            if x_max + 1 < BOARD_SIZE && self.mapa_conhecimento[x_max + 1][y] == EstadoCelula::Desconhecido {
                probabilidades[x_max + 1][y] *= 100.0;
                godot_print!("IA Difícil: ⬇️ Prioridade MÁXIMA em ({}, {})", x_max + 1, y);
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

    /// Encontra a sequência LINEAR de acertos conectada à célula (x, y)
    /// Navios são sempre lineares (horizontal OU vertical), nunca em L ou T
    fn encontrar_sequencia_conectada(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut sequencia = vec![(x, y)];
        
        // Verificar se há acertos adjacentes em cada direção
        let esquerda = y > 0 && self.mapa_conhecimento[x][y - 1] == EstadoCelula::Acerto;
        let direita = y + 1 < BOARD_SIZE && self.mapa_conhecimento[x][y + 1] == EstadoCelula::Acerto;
        let cima = x > 0 && self.mapa_conhecimento[x - 1][y] == EstadoCelula::Acerto;
        let baixo = x + 1 < BOARD_SIZE && self.mapa_conhecimento[x + 1][y] == EstadoCelula::Acerto;
        
        let tem_horizontal = esquerda || direita;
        let tem_vertical = cima || baixo;
        
        // Se tem ambos, é uma interseção - precisamos determinar qual navio afundou
        // Contamos qual direção tem a sequência mais longa
        let direcao_horizontal = if tem_horizontal && tem_vertical {
            let mut count_h = 1;
            let mut cy = y;
            while cy > 0 && self.mapa_conhecimento[x][cy - 1] == EstadoCelula::Acerto {
                count_h += 1;
                cy -= 1;
            }
            cy = y;
            while cy + 1 < BOARD_SIZE && self.mapa_conhecimento[x][cy + 1] == EstadoCelula::Acerto {
                count_h += 1;
                cy += 1;
            }
            
            let mut count_v = 1;
            let mut cx = x;
            while cx > 0 && self.mapa_conhecimento[cx - 1][y] == EstadoCelula::Acerto {
                count_v += 1;
                cx -= 1;
            }
            cx = x;
            while cx + 1 < BOARD_SIZE && self.mapa_conhecimento[cx + 1][y] == EstadoCelula::Acerto {
                count_v += 1;
                cx += 1;
            }
            
            count_h >= count_v  // Prioriza horizontal em caso de empate
        } else {
            tem_horizontal
        };
        
        if direcao_horizontal {
            // Expandir horizontalmente (esquerda e direita)
            let mut cy = y;
            while cy > 0 && self.mapa_conhecimento[x][cy - 1] == EstadoCelula::Acerto {
                cy -= 1;
                sequencia.push((x, cy));
            }
            
            cy = y;
            while cy + 1 < BOARD_SIZE && self.mapa_conhecimento[x][cy + 1] == EstadoCelula::Acerto {
                cy += 1;
                sequencia.push((x, cy));
            }
        } else if tem_vertical {
            // Expandir verticalmente (cima e baixo)
            let mut cx = x;
            while cx > 0 && self.mapa_conhecimento[cx - 1][y] == EstadoCelula::Acerto {
                cx -= 1;
                sequencia.push((cx, y));
            }
            
            cx = x;
            while cx + 1 < BOARD_SIZE && self.mapa_conhecimento[cx + 1][y] == EstadoCelula::Acerto {
                cx += 1;
                sequencia.push((cx, y));
            }
        }
        // Se não tem adjacentes, é um navio de tamanho 1 (submarino)
        
        sequencia
    }
}

impl EstrategiaIA for EstrategiaDificil {
    fn escolher_alvo(&mut self, _tabuleiro: &EstadoTabuleiro) -> Option<(usize, usize)> {
        // 🎯 FASE 1: Prioridade ABSOLUTA - atacar navios lineares já encontrados
        if !self.acertos_ativos.is_empty() {
            let grupos = self.encontrar_grupos_lineares();
            
            // Procurar grupos de 2+ células (navios lineares confirmados) com pontas disponíveis
            let grupos_lineares_com_pontas: Vec<_> = grupos.iter()
                .filter(|g| g.len() >= 2)
                .filter(|g| self.grupo_tem_pontas_disponiveis(g))
                .collect();
            
            if !grupos_lineares_com_pontas.is_empty() {
                // Escolher o grupo maior (mais células = mais comprometimento)
                let grupo_escolhido = grupos_lineares_com_pontas.iter()
                    .max_by_key(|g| g.len())
                    .unwrap();
                
                godot_print!("IA Difícil: 🎯 NAVIO LINEAR DETECTADO! Grupo: {:?} (tamanho {})", grupo_escolhido, grupo_escolhido.len());
                
                // Obter as pontas disponíveis
                let pontas = self.obter_pontas_disponiveis(grupo_escolhido);
                
                if !pontas.is_empty() {
                    // Escolher uma ponta aleatória
                    let mut rng = godot::classes::RandomNumberGenerator::new_gd();
                    rng.randomize();
                    let idx = rng.randi_range(0, (pontas.len() - 1) as i32) as usize;
                    let alvo = pontas[idx];
                    
                    godot_print!("IA Difícil: 🔥 ATAQUE DIRETO na ponta: {:?}", alvo);
                    return Some(alvo);
                }
            }
        }
        
        // 🎯 FASE 2: Sistema de probabilidades (caça ou exploração)
        godot_print!("IA Difícil: 🧮 Usando sistema de probabilidades");
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
                // Adiciona o último acerto antes de processar
                if !self.todos_acertos.contains(&(x, y)) {
                    self.todos_acertos.push((x, y));
                }
                
                // Temporariamente marca como Acerto para encontrar a sequência
                self.mapa_conhecimento[x][y] = EstadoCelula::Acerto;

                // 🎯 A IA "vê" visualmente qual navio afundou!
                // Encontra a sequência LINEAR de acertos conectada
                let sequencia = self.encontrar_sequencia_conectada(x, y);
                let tamanho_calculado = sequencia.len();

                godot_print!("IA Difícil: 👁️ Analisando visual... Sequência detectada: {:?} (tamanho {})", 
                    sequencia, tamanho_calculado);

                // Marca todas as células da sequência como Afundado
                for &(sx, sy) in &sequencia {
                    self.mapa_conhecimento[sx][sy] = EstadoCelula::Afundado;
                }

                // 🔊 Confirma pelo som que o navio foi completamente destruído
                if let Some(count) = self.navios_restantes.get_mut(&tamanho_calculado) {
                    if *count > 0 {
                        *count -= 1;
                        godot_print!("IA Difícil: ✅ {} afundado confirmado! Tamanho: {}. Restantes: {:?}", 
                            nome_navio, tamanho_calculado, self.navios_restantes);
                    }
                }

                // Remove APENAS os acertos da sequência afundada da lista ativa
                // Outros acertos (de navios diferentes) permanecem
                for &pos in &sequencia {
                    self.acertos_ativos.retain(|&p| p != pos);
                }
                
                godot_print!("IA Difícil: 🎯 Acertos ativos restantes: {:?}", self.acertos_ativos);
            }
            _ => {}
        }
    }
}
