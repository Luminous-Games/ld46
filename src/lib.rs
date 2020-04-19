#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

use na::Vector2;
use wasm_bindgen::__rt::std::collections::hash_map::RandomState;
use wasm_bindgen::__rt::std::collections::HashSet;
use wasm_bindgen::prelude::*;

use engine::key::{key_codes, KeyManager};
use engine::renderer::Renderer;
use engine::{Renderable, World};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

trait Collidable {
    fn get_pos(&self) -> &na::Vector2<f32>;
    fn get_range(&self) -> f32;

    fn collide(&self, pos: &na::Vector2<f32>, speed: &na::Vector2<f32>) -> na::Vector2<f32> {
        let mut speed = speed.clone_owned();
        if (self.get_pos() - pos).norm() < self.get_range() {
            if pos.x > self.get_pos().x {
                speed.x = f32::max(0.0, speed.x);
            } else {
                speed.x = f32::min(0.0, speed.x);
            }
            if pos.y > self.get_pos().y {
                speed.y = f32::max(0.0, speed.y);
            } else {
                speed.y = f32::min(0.0, speed.y);
            }
            // player.speed = (player_pos - fire_pos).scale(0.1);
        };
        speed
    }
}

struct SomeWorld {
    renderables: Vec<Rc<RefCell<dyn Renderable>>>,
    collidables: Vec<Rc<RefCell<dyn Collidable>>>,
    player: Rc<RefCell<Player>>,
    fire: Rc<RefCell<Fire>>,
    last_tick: f64,
}

impl SomeWorld {
    fn new(player: Player) -> SomeWorld {
        let player_in_a_box = Rc::new(RefCell::new(player));
        let fire_in_a_box = Rc::new(RefCell::new(Fire {
            pos: na::Vector2::new(300.0, 300.0),
            index: 0,
        }));
        let tree_in_a_box = Rc::new(RefCell::new(Tree {
            pos: na::Vector2::new(400.0, 100.0),
            index: 0,
        }));
        SomeWorld {
            renderables: vec![
                tree_in_a_box.clone(),
                fire_in_a_box.clone(),
                player_in_a_box.clone(),
            ],
            collidables: vec![fire_in_a_box.clone(), tree_in_a_box.clone()],
            player: player_in_a_box.clone(),
            fire: fire_in_a_box.clone(),
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

impl engine::World for SomeWorld {
    fn tick<'a>(
        &'a mut self,
        key_manager: &KeyManager,
        timestamp: f64,
    ) -> Vec<Rc<RefCell<dyn Renderable>>> {
        let mut player = self.player.borrow_mut();
        let direction = SomeWorld::get_direction(key_manager);
        player.speed += direction.scale((timestamp - self.last_tick) as f32 * 0.1);
        let norm = player.speed.norm();
        if norm > 10.0 {
            player.speed.scale_mut(10.0 / norm);
        }
        player.speed.scale_mut(0.8);
        for collidable in &self.collidables {
            player.speed = collidable.borrow().collide(&player.pos, &player.speed)
        }
        let speed = player.speed.clone();
        player.pos += speed.scale((timestamp - self.last_tick) as f32 * 0.05);
        self.last_tick = timestamp;
        self.renderables.clone()
    }
}

#[derive(Clone)]
struct Player {
    pos: na::Vector2<f32>,
    speed: na::Vector2<f32>,
    index: u32,
}

impl Renderable for Player {
    fn render(&self, r: &mut Renderer) {
        let tm = engine::renderer::TextureMap::new(2, 1);
        r.draw_quad(
            na::Vector2::new(self.pos.x, self.pos.y),
            na::Vector2::new(128.0, 128.0),
            tm.get_texture(1, 0),
        );
    }
}
#[derive(Clone)]
struct Fire {
    pos: na::Vector2<f32>,
    index: u32,
}

impl Renderable for Fire {
    fn render(&self, r: &mut Renderer) {
        let tm = engine::renderer::TextureMap::new(2, 1);
        r.draw_quad(
            na::Vector2::new(self.pos.x, self.pos.y + 32.0),
            na::Vector2::new(64.0, 64.0),
            tm.get_texture(0, 0),
        );
    }
}

impl Collidable for Fire {
    fn get_pos(&self) -> &Vector2<f32> {
        &self.pos
    }

    fn get_range(&self) -> f32 {
        32.0
    }
}

#[derive(Clone)]
struct Tree {
    pos: na::Vector2<f32>,
    index: u32,
}

impl Renderable for Tree {
    fn render(&self, r: &mut Renderer) {
        let tm = engine::renderer::TextureMap::new(2, 1);
        r.draw_quad(
            na::Vector2::new(self.pos.x, self.pos.y),
            na::Vector2::new(128.0, 128.0),
            tm.get_texture(1, 0),
        );
    }
}

impl Collidable for Tree {
    fn get_pos(&self) -> &Vector2<f32> {
        &self.pos
    }

    fn get_range(&self) -> f32 {
        32.0
    }
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    let player = Player {
        pos: na::Vector2::new(100.0, 100.0),
        speed: na::Vector2::zeros(),
        index: 0,
    };
    engine::start(Box::new(SomeWorld::new(player)) as Box<dyn World>);
}
