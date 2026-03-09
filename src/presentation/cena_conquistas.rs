use godot::prelude::*;
use godot::classes::{Button, Control, IControl, Label, VBoxContainer};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct CenaConquistas {
    base: Base<Control>,
}

#[godot_api]
impl IControl for CenaConquistas {
    fn init(base: Base<Control>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        let conquistas_desbloqueadas = self.obter_conquistas_usuario();
        self.popular_lista(conquistas_desbloqueadas);

        let mut btn_voltar = self.base().get_node_as::<Button>("botao_voltar");
        let callable = self.base().callable("voltar_menu");
        btn_voltar.connect("pressed", &callable);
    }
}

#[godot_api]
impl CenaConquistas {
    fn obter_conquistas_usuario(&self) -> Vec<String> {
        let mut login_atual = String::new();

        if godot::classes::FileAccess::file_exists("res://dados/usuario_atual.json") {
            let conteudo = godot::classes::FileAccess::get_file_as_string("res://dados/usuario_atual.json").to_string();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&conteudo) {
                if let Some(login) = json.get("login").and_then(|v| v.as_str()) {
                    login_atual = login.to_string();
                }
            }
        } else if godot::classes::FileAccess::file_exists("res://usuario_atual.json") {
            let conteudo = godot::classes::FileAccess::get_file_as_string("res://usuario_atual.json").to_string();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&conteudo) {
                if let Some(login) = json.get("login").and_then(|v| v.as_str()) {
                    login_atual = login.to_string();
                }
            }
        }

        if login_atual.is_empty() {
            return vec![];
        }

        let mut conquistas = Vec::new();
        let mut json_string = String::new();

        if godot::classes::FileAccess::file_exists("res://dados/usuarios.json") {
            json_string = godot::classes::FileAccess::get_file_as_string("res://dados/usuarios.json").to_string();
        } else if godot::classes::FileAccess::file_exists("res://usuarios.json") {
            json_string = godot::classes::FileAccess::get_file_as_string("res://usuarios.json").to_string();
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_string) {
            if let Some(array) = json.as_array() {
                for item in array {
                    let login = item.get("login").and_then(|v| v.as_str()).unwrap_or("");
                    if login == login_atual {
                        if let Some(conquistas_array) = item.get("conquistas").and_then(|v| v.as_array()) {
                            for c in conquistas_array {
                                if let Some(c_str) = c.as_str() {
                                    conquistas.push(c_str.to_string());
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        conquistas
    }

    fn popular_lista(&mut self, desbloqueadas: Vec<String>) {
        let mut lista_container = self.base().get_node_as::<VBoxContainer>("VBoxContainer");

        let todas_conquistas = vec![
            ("Almirante", "Vencer sem perder navios"),
            ("Capitao", "Acertar 7 tiros seguidos"),
            ("CapitaoDeMarEGuerra", "Acertar 8 tiros seguidos"),
            ("Marinheiro", "Vencer em 20 rodadas ou menos"),
        ];

        for (nome, descricao) in todas_conquistas {
            let mut label = Label::new_alloc();

            let mut texto = format!("{} - {}", nome, descricao);
            if desbloqueadas.contains(&nome.to_string()) {
                texto = format!("✅ {}", texto);
                label.set_modulate(Color::from_rgb(1.0, 0.84, 0.0));
            } else {
                texto = format!("🔒 {}", texto);
                label.set_modulate(Color::from_rgb(0.5, 0.5, 0.5));
            }

            label.set_text(&texto);
            lista_container.add_child(&label);
        }
    }

    #[func]
    fn voltar_menu(&mut self) {
        let mut tree = self.base().get_tree();
        tree.change_scene_to_file("res://MenuPrincipal.tscn");
    }
}