#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate wee_alloc;
use engine::key::{key_codes, KeyManager};
use engine::renderer::Renderer;
use engine::{Collider, GameObject, Rend, World};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

struct TexturedBox {
    size: na::Vector2<f32>,
    texture: engine::renderer::Texture,
}

impl Rend for TexturedBox {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.draw_quad(game_object.pos, self.size, &self.texture)
    }
}

struct SomeWorld<'a> {
    game_objects: HashMap<&'a str, GameObject>,
    last_tick: f64,
}

impl<'a> SomeWorld<'a> {
    fn new() -> SomeWorld<'a> {
        let texture_map = engine::renderer::TextureMap::new(2, 1);

        let mut player = GameObject::new(na::Point2::new(100.0, 100.0));
        player.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(128.0, 128.0),
            texture: texture_map.get_texture(1, 0),
        }));
        let mut fire = GameObject::new(na::Point2::new(300.0, 300.0));
        fire.add_collider(Collider::new(32.0));
        fire.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(64.0, 64.0),
            texture: texture_map.get_texture(0, 0),
        }));
        let mut tree = GameObject::new(na::Point2::new(400.9, 100.0));
        tree.add_collider(Collider::new(16.0));
        tree.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(128.0, 128.0),
            texture: texture_map.get_texture(1, 0),
        }));
        let mut game_objects = HashMap::new();
        game_objects.insert("player", player);
        game_objects.insert("fire", fire);
        game_objects.insert("tree", tree);
        SomeWorld {
            game_objects,
            last_tick: 0.0,
        }
    }

    fn get_direction(key_manager: &KeyManager) -> na::Vector2<f32> {
        let mut direction = na::Vector2::zeros();
        if key_manager.key_pressed(key_codes::W) || key_manager.key_pressed(key_codes::UP_ARROW) {
            direction.y += 1.0;
        }
        if key_manager.key_pressed(key_codes::S) || key_manager.key_pressed(key_codes::DOWN_ARROW) {
            direction.y += -1.0;
        }
        if key_manager.key_pressed(key_codes::D) || key_manager.key_pressed(key_codes::RIGHT_ARROW)
        {
            direction.x += 1.0;
        }
        if key_manager.key_pressed(key_codes::A) || key_manager.key_pressed(key_codes::LEFT_ARROW) {
            direction.x += -1.0;
        }
        direction
    }
}

impl<'a> engine::World for SomeWorld<'a> {
    fn tick(&mut self, key_manager: &KeyManager, timestamp: f64) {
        let direction = SomeWorld::get_direction(key_manager);

        let player = self.game_objects.get("player").unwrap();
        let mut speed = player.speed.clone();
        speed += direction.scale((timestamp - self.last_tick) as f32 * 0.1);
        let norm = speed.norm();
        if norm > 10.0 {
            speed.scale_mut(10.0 / norm);
        }
        speed.scale_mut(0.8);

        for game_object in self.game_objects.values() {
            let collider = game_object.get_collider();
            match collider {
                Some(collider) => speed = collider.collide(&game_object, &player.pos, &speed),
                None => (),
            }
        }

        let mut player = self.game_objects.get_mut("player").unwrap();
        player.pos += speed.scale((timestamp - self.last_tick) as f32 * 0.05);
        player.speed = speed;
        self.last_tick = timestamp;
    }

    fn get_game_objects(&self) -> Vec<&GameObject> {
        self.game_objects.values().collect()
    }
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    log!("Starting game!");
    engine::start(Box::new(SomeWorld::new()) as Box<dyn World>).unwrap();
}
