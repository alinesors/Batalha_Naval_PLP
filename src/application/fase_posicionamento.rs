use crate::domain::jogador::Jogador;
use crate::domain::tabuleiro::{FROTA_PADRAO, BOARD_SIZE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewPosicionamento {
    pub celulas: Vec<(usize, usize)>,
    pub valido: bool,
}

pub struct FasePosicionamento {
    fila_navios: Vec<(String, usize)>,
    navio_selecionado: Option<usize>,
    navio_em_reposicionamento: Option<(String, usize)>,
    orientacao_horizontal: bool,
    modo_edicao: bool,
}

impl FasePosicionamento {
    pub fn nova() -> Self {
        let mut fila_navios = Vec::new();
        for config in FROTA_PADRAO.iter() {
            for _ in 0..config.quantidade {
                fila_navios.push((config.nome.to_string(), config.tamanho));
            }
        }

        Self {
            fila_navios,
            navio_selecionado: None,
            navio_em_reposicionamento: None,
            orientacao_horizontal: true,
            modo_edicao: false,
        }
    }

    pub fn alternar_orientacao(&mut self) {
        self.orientacao_horizontal = !self.orientacao_horizontal;
    }

    pub fn orientacao_texto(&self) -> &'static str {
        if self.orientacao_horizontal {
            "Horizontal"
        } else {
            "Vertical"
        }
    }

    pub fn navio_atual(&self) -> Option<(&str, usize)> {
        // Priorizar navio em reposicionamento
        if let Some((nome, tamanho)) = &self.navio_em_reposicionamento {
            return Some((nome.as_str(), *tamanho));
        }
        
        // Senão, pegar da fila
        match self.navio_selecionado {
            Some(idx) => self.fila_navios
                .get(idx)
                .map(|(nome, tamanho)| (nome.as_str(), *tamanho)),
            None => None,
        }
    }

    pub fn selecionar_navio(&mut self, indice: usize) -> bool {
        // Não permitir selecionar outro navio se há um em reposicionamento
        if self.navio_em_reposicionamento.is_some() {
            return false;
        }
        
        if indice < self.fila_navios.len() {
            self.navio_selecionado = Some(indice);
            true
        } else {
            false
        }
    }

    pub fn navio_selecionado_indice(&self) -> Option<usize> {
        self.navio_selecionado
    }

    pub fn obter_fila_navios(&self) -> &[(String, usize)] {
        &self.fila_navios
    }

    fn ajustar_coordenada_para_centro(&self, x: usize, y: usize, tamanho: usize) -> (usize, usize) {
        let offset = tamanho / 2;
        if self.orientacao_horizontal {
            (x, y.saturating_sub(offset))
        } else {
            (x.saturating_sub(offset), y)
        }
    }

    pub fn preview_na_posicao(
        &self,
        jogador: &Jogador,
        x: usize,
        y: usize,
    ) -> Option<PreviewPosicionamento> {
        let (_, tamanho) = self.navio_atual()?;

        let (start_x, start_y) = self.ajustar_coordenada_para_centro(x, y, tamanho);
        let mut celulas = Vec::with_capacity(tamanho);

        for i in 0..tamanho {
            let (cx, cy) = if self.orientacao_horizontal {
                (start_x as i32, start_y as i32 + i as i32)
            } else {
                (start_x as i32 + i as i32, start_y as i32)
            };

            if cx >= 0 && cy >= 0 && cx < BOARD_SIZE as i32 && cy < BOARD_SIZE as i32 {
                celulas.push((cx as usize, cy as usize));
            }
        }

        let valido = celulas.len() == tamanho
            && jogador
                .tabuleiro()
                .pode_posicionar_navio(start_x, start_y, tamanho, self.orientacao_horizontal);

        Some(PreviewPosicionamento { celulas, valido })
    }

    pub fn tentar_posicionar_navio(
        &mut self,
        jogador: &mut Jogador,
        x: usize,
        y: usize,
    ) -> Result<bool, String> {
        // Obter nome e tamanho do navio (priorizar navio em reposicionamento)
        let (nome, tamanho) = if let Some((nome, tamanho)) = &self.navio_em_reposicionamento {
            (nome.clone(), *tamanho)
        } else if let Some(idx_selecionado) = self.navio_selecionado {
            let Some((nome, tamanho)) = self.fila_navios.get(idx_selecionado).cloned() else {
                return Ok(true);
            };
            (nome, tamanho)
        } else {
            return Err("Nenhum navio selecionado".into());
        };

        let (start_x, start_y) = self.ajustar_coordenada_para_centro(x, y, tamanho);

        jogador
            .tabuleiro_mut()
            .posicionar_navio(&nome, start_x, start_y, tamanho, self.orientacao_horizontal)?;

        // Limpar navio em reposicionamento (se houver)
        if self.navio_em_reposicionamento.is_some() {
            self.navio_em_reposicionamento = None;
        } else if let Some(idx_selecionado) = self.navio_selecionado {
            // Remover o navio da fila
            self.fila_navios.remove(idx_selecionado);
        }
        
        self.navio_selecionado = None;
        
        Ok(self.terminou())
    }

    pub fn terminou(&self) -> bool {
        self.fila_navios.is_empty()
    }

    pub fn todos_posicionados(&self) -> bool {
        self.fila_navios.is_empty()
    }

    pub fn em_modo_edicao(&self) -> bool {
        self.modo_edicao
    }

    pub fn ativar_modo_edicao(&mut self) {
        self.modo_edicao = true;
    }

    pub fn desativar_modo_edicao(&mut self) {
        self.modo_edicao = false;
    }

    pub fn remover_navio(&mut self, nome: &str) -> bool {
        // Encontrar o navio na frota original para pegar o tamanho
        let tamanho_navio = FROTA_PADRAO
            .iter()
            .find(|config| config.nome == nome)
            .map(|config| config.tamanho);

        if let Some(tamanho) = tamanho_navio {
            // Guardar navio em reposicionamento (não adiciona à fila)
            self.navio_em_reposicionamento = Some((nome.to_string(), tamanho));
            self.navio_selecionado = None;
            return true;
        }

        false
    }
}
