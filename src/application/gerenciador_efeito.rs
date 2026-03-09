use godot::classes::{AnimatedSprite2D, AtlasTexture, Node, Node2D, ResourceLoader, SpriteFrames, Texture2D};
use godot::prelude::*;

const SMOKE_TEXTURE_PATH: &str = "res://textures/smoke.png";
const FRAME_W: f32 = 16.0;
const FRAME_H: f32 = 16.0;

const FRAME_COUNT: i32 = 8;
const ANIMATION_FPS: f64 = 10.0;
const ANIM_NAME: &str = "smoke";

pub struct GerenciadorEfeito {
    particulas: Vec<Gd<AnimatedSprite2D>>,
    textura: Option<Gd<Texture2D>>,
}

impl GerenciadorEfeito {
    pub fn novo() -> Self {
        Self {
            particulas: Vec::new(),
            textura: None,
        }
    }

    fn obter_textura(&mut self) -> Option<Gd<Texture2D>> {
        if self.textura.is_none() {
            let mut loader = ResourceLoader::singleton();
            self.textura = loader
                .load(SMOKE_TEXTURE_PATH)
                .and_then(|r| r.try_cast::<Texture2D>().ok());
            if self.textura.is_none() {
                godot_warn!("GerenciadorEfeito: textura não encontrada em '{}'", SMOKE_TEXTURE_PATH);
            }
        }
        self.textura.clone()
    }


    pub fn disparar_fumaca(&mut self, pai: Gd<Node2D>, pos_global: Vector2) {
        let Some(textura) = self.obter_textura() else {
            return;
        };

        let mut frames = SpriteFrames::new_gd();
        frames.add_animation(ANIM_NAME);
        frames.set_animation_loop(ANIM_NAME, false);
        frames.set_animation_speed(ANIM_NAME, ANIMATION_FPS);

        for i in 0..FRAME_COUNT {
            let mut atlas = AtlasTexture::new_gd();
            atlas.set_atlas(&textura);
            atlas.set_region(Rect2::new(
                Vector2::new(i as f32 * FRAME_W, 0.0),
                Vector2::new(FRAME_W, FRAME_H),
            ));

            frames.add_frame(ANIM_NAME, &atlas.upcast::<Texture2D>());
        }

        let mut sprite = AnimatedSprite2D::new_alloc();
        sprite.set_sprite_frames(&frames);
        sprite.set_z_index(200); // acima de todos os outros layers
        sprite.set_scale(Vector2::new(1.0, 1.0));

        let pos_local = pai.to_local(pos_global);
        sprite.set_position(pos_local);

        let mut pai_mut = pai;
        pai_mut.add_child(&sprite.clone().upcast::<Node>());
        sprite.play_ex().name(ANIM_NAME).done();

        self.particulas.push(sprite);
    }

    pub fn atualizar(&mut self) {
        self.particulas.retain_mut(|sprite| {
            if sprite.is_playing() {
                true
            } else {
                sprite.queue_free();
                false
            }
        });
    }
}

pub fn posicao_global_tile(
    tilemap: &godot::classes::TileMapLayer,
    map_coord: Vector2i,
) -> Vector2 {
    let local = tilemap.map_to_local(map_coord);
    tilemap.to_global(local)
}
