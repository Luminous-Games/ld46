#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate wee_alloc;

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use engine::key::{key_codes, KeyManager};
use engine::renderer::{Renderer, TextureMap};
use engine::{Collider, GameObject, Rend, World};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct TexturedBox {
    size: na::Vector2<f32>,
    texture: engine::renderer::Texture,
}

impl Rend for TexturedBox {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.draw_quad(game_object.pos, self.size, &self.texture)
    }
}

struct Thermometer {
    size: na::Vector2<f32>,
    body_pos: (f32, f32),
    filling_pos: (f32, f32),
    texture_size: (f32, f32),
    texture_map: TextureMap,
    temperature: f32,
}

impl Thermometer {
    pub fn new(
        body_pos: (f32, f32),
        filling_pos: (f32, f32),
        texture_size: (f32, f32),
        texture_map: TextureMap,
    ) -> Thermometer {
        Thermometer {
            size: na::Vector2::new(128.0, 32.0),
            body_pos,
            filling_pos,
            texture_size,
            texture_map,
            temperature: 0.5,
        }
    }
}

impl Rend for Thermometer {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.draw_quad(
            game_object.pos,
            self.size,
            &self.texture_map.get_texture_custom(
                self.body_pos.0,
                self.body_pos.1,
                self.texture_size.0,
                self.texture_size.1,
            ),
        );
        let mut filling_size = self.size.clone_owned();
        filling_size.x *= self.temperature;
        let mut filling_pos = game_object.pos.clone();
        filling_pos.x -= self.size.x * 0.5 * (1.0 - self.temperature);
        renderer.draw_quad(
            filling_pos,
            filling_size,
            &self.texture_map.get_texture_custom(
                self.filling_pos.0,
                self.filling_pos.1,
                self.texture_size.0 * self.temperature,
                self.texture_size.1,
            ),
        );
    }
}

struct Cam {}

impl Rend for Cam {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject) {
        renderer.set_camera(game_object.pos);
    }
}

struct SomeWorld<'a> {
    game_objects: HashMap<&'a str, GameObject>,
    last_tick: f64,
}

impl<'a> SomeWorld<'a> {
    fn new() -> SomeWorld<'a> {
        let spritesheet = engine::renderer::TextureMap::new(4, 1, "spritesheet".to_string());

        let mut player = GameObject::new(na::Point2::new(0.0, 0.0));
        player.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(128.0, 128.0),
            texture: spritesheet.get_texture(1, 0),
        }));
        player.add_rend(Box::new(Cam {}));
        let mut fire = GameObject::new(na::Point2::new(300.0, 300.0));
        fire.add_collider(Collider::new(32.0));
        fire.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(64.0, 64.0),
            texture: spritesheet.get_texture(0, 0),
        }));
        let mut tree = GameObject::new(na::Point2::new(400.9, 100.0));
        tree.add_collider(Collider::new(16.0));
        tree.add_rend(Box::new(TexturedBox {
            size: na::Vector2::new(128.0, 128.0),
            texture: spritesheet.get_texture(1, 0),
        }));

        let mut thermometer = GameObject::new(na::Point2::new(100.0, 50.0));
        thermometer.add_rend(Box::new(Thermometer::new(
            (2.0, 0.0),
            (2.0, 0.5),
            (2.0, 0.5),
            spritesheet.clone(),
        )));

        let mut game_objects = HashMap::new();
        game_objects.insert("player", player);
        game_objects.insert("fire", fire);
        game_objects.insert("tree", tree);
        game_objects.insert("thermometer", thermometer);
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
        let fire = self.game_objects.get("fire").unwrap();

        let mut speed = player.speed.clone();
        speed += direction * ((timestamp - self.last_tick) as f32 * 0.1);
        let norm = speed.norm();
        if norm > 10.0 {
            speed *= 10.0 / norm;
        }
        speed *= 0.8;

        for game_object in self.game_objects.values() {
            let collider = game_object.get_collider();
            match collider {
                Some(collider) => speed = collider.collide(&game_object, &player.pos, &speed),
                None => (),
            }
        }

        self.game_objects.get_mut("thermometer").unwrap().rend[0]
            .downcast_mut::<Thermometer>()
            .unwrap()
            .temperature = f32::min(
            1.0,
            f32::max(0.0, 1.0 - ((player.pos - fire.pos).norm() - 48.0) / 200.0),
        );

        let mut player = self.game_objects.get_mut("player").unwrap();
        player.pos += speed * (timestamp - self.last_tick) as f32 * 0.05;
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
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Game starting");
    engine::start(Box::new(SomeWorld::new()) as Box<dyn World>).unwrap();
}
