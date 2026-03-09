use godot::prelude::*;
use crate::application::services::usuario_service::UsuarioService;
use crate::domain::entidades::conquista::Conquista;
use crate::infrastructure::repositorio_usuario_json::RepositorioUsuarioJson;
use crate::domain::entidades::usuario::Usuario;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct UsuarioNode {
    service: UsuarioService<RepositorioUsuarioJson>,
    base: Base<Node>
}

fn usuario_para_dict(u : Usuario) -> Dictionary<GString, Variant> {
    let mut dict = Dictionary::new();
    dict.set("id", u.id as i64);
    dict.set("nome", &GString::from(&u.nome));
    dict.set("login", &GString::from(&u.login));
    dict.set("jogos_totais", u.jogos_totais as i64);
    dict.set("vitorias", u.vitorias as i64);
    dict.set("derrotas", u.derrotas as i64);
    dict.set("taxa_de_vitoria", u.taxa_de_vitoria() as f64);
    let mut conquistas = PackedStringArray::new();
    for conquista in u.conquistas {
        conquistas.push(&GString::from(conquista_para_str(&conquista)));
    }
    dict.set("conquistas", &conquistas);
    dict
}

fn conquista_para_str(conquista: &Conquista) -> &'static str {
    match conquista {
        Conquista::Almirante => "Almirante",
        Conquista::Capitao => "Capitao",
        Conquista::CapitaoDeMarEGuerra => "CapitaoDeMarEGuerra",
        Conquista::Marinheiro => "Marinheiro",
    }
}

#[godot_api]
impl INode for UsuarioNode {
    fn init(base: Base<Node>) -> Self {
        Self{
            service: UsuarioService {
                repo: RepositorioUsuarioJson::new("dados/usuarios.json")
            },
            base
        }
    }
}

#[godot_api]
impl UsuarioNode {

    #[func]
    pub fn registrar(&mut self, nome: GString, login: GString, senha: GString) -> bool {
        self.service
            .registrar(nome.to_string(), login.to_string(), senha.to_string())
            .is_ok()
    }

    #[func]
    pub fn login(&self, login: GString, senha: GString) -> Dictionary<GString, Variant> {
        match self.service.login(&login.to_string(), &senha.to_string()) {
            Ok(usuario) => usuario_para_dict(usuario),
            Err(_) => Dictionary::new()
        }
    }

    #[func]
    pub fn buscar_por_login(&self, login: GString) -> Dictionary<GString, Variant> {
        match self.service.buscar_por_login(&login.to_string()) {
            Ok(usuario) => usuario_para_dict(usuario),
            Err(_) => Dictionary::new()
        }
    }

    #[func]
    pub fn atualizar_nome(&mut self, login: GString, novo_nome: GString) -> bool {
        self.service
            .atualizar_nome(&login.to_string(), novo_nome.to_string())
            .is_ok()
    }

    #[func]
    pub fn atualizar_senha(&mut self, login: GString, senha_atual: GString, nova_senha: GString) -> bool {
        self.service
            .atualizar_senha(&login.to_string(), &senha_atual.to_string(), nova_senha.to_string())
            .is_ok()
    }

    #[func]
    pub fn excluir_conta(&mut self, login: GString, senha: GString) -> bool {
        self.service
            .excluir_conta(&login.to_string(), &senha.to_string())
            .is_ok()
    }

    #[func]
    pub fn adicionar_conquista(&mut self, login: GString, conquista: GString) -> bool {
        let conquista = match conquista.to_string().as_str() {
            "Almirante" => Conquista::Almirante,
            "Capitao" => Conquista::Capitao,
            "CapitaoDeMarEGuerra" => Conquista::CapitaoDeMarEGuerra,
            "Marinheiro" => Conquista::Marinheiro,
            _ => return false
        };

        self.service
            .adicionar_conquista(&login.to_string(), conquista)
            .is_ok()
    }

    #[func]
    pub fn listar_conquistas(&self, login: GString) -> PackedStringArray {
        match self.service.listar_conquistas(&login.to_string()) {
            Ok(conquistas) => {
                let mut array = PackedStringArray::new();
                for c in conquistas {
                    array.push(&GString::from(conquista_para_str(&c)));
                }
                array
            },
            Err(_) => PackedStringArray::new()
        }
    }

    #[func]
    pub fn registrar_vitoria(&mut self, login: GString) -> bool {
        self.service
            .registrar_vitoria(&login.to_string())
            .is_ok()
    }

    #[func]
    pub fn registrar_derrota(&mut self, login: GString) -> bool {
        self.service
            .registrar_derrota(&login.to_string())
            .is_ok()
    }

    #[func]
    pub fn obter_estatisticas(&self, login: GString) -> Dictionary<GString, Variant> {
        match self.service.obter_estatisticas(&login.to_string()) {
            Ok((jogos_totais, vitorias, derrotas, taxa)) => {
                let mut dict = Dictionary::new();
                dict.set("jogos_totais", jogos_totais as i64);
                dict.set("vitorias", vitorias as i64);
                dict.set("derrotas", derrotas as i64);
                dict.set("taxa_de_vitoria", taxa as f64);
                dict
            },
            Err(_) => Dictionary::new()
        }
    }
}