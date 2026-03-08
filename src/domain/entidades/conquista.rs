use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Conquista {
    Almirante, // Vencer sem perder navios
    Capitao, // Acertar 7 tiros seguidos
    CapitaoDeMarEGuerra, // Acertar 8 tiros seguidos
    Marinheiro // Vencer em X tempo
}