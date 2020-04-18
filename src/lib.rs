#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

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

struct SomeWorld {
    renderables: Vec<Rc<RefCell<dyn engine::Renderable>>>,
    player: Rc<RefCell<Player>>,
    fire: Rc<RefCell<Fire>>,
    last_tick: f64,
}

#[derive(Clone)]
struct Player {
    pos: na::Vector2<f32>,
    speed: na::Vector2<f32>,
}

impl Renderable for Player {
    fn render(&self, r: &mut Renderer) {
        let tm = engine::renderer::TextureMap::new(1, 1);
        r.draw_quad(
            na::Vector2::new(self.pos.x, self.pos.y),
            na::Vector2::new(128.0, 128.0),
            tm.get_texture(0, 0),
        );
    }
}

#[derive(Clone)]
struct Fire {
    pos: na::Vector2<f32>,
}

impl Renderable for Fire {
    fn render(&self, r: &mut Renderer) {
        let tm = engine::renderer::TextureMap::new(1, 1);
        r.draw_quad(
            na::Vector2::new(self.pos.x, self.pos.y + 32.0),
            na::Vector2::new(64.0, -64.0),
            tm.get_texture(0, 0),
        );
    }
}

impl SomeWorld {
    fn new(player: Player) -> SomeWorld {
        let player_in_a_box = Rc::new(RefCell::new(player));
        let fire_in_a_box = Rc::new(RefCell::new(Fire {
            pos: na::Vector2::new(300.0, 300.0),
        }));
        SomeWorld {
            renderables: vec![fire_in_a_box.clone(), player_in_a_box.clone()],
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
        let player_pos = &player.pos.clone();
        let fire_pos = &self.fire.borrow().pos.clone();
        if (player_pos - fire_pos).norm() < 32.0 {
            if player_pos.x > fire_pos.x {
                player.speed.x = f32::max(0.0, player.speed.x);
            } else {
                player.speed.x = f32::min(0.0, player.speed.x);
            }
            if player_pos.y > fire_pos.y {
                player.speed.y = f32::max(0.0, player.speed.y);
            } else {
                player.speed.y = f32::min(0.0, player.speed.y);
            }
            // player.speed = (player_pos - fire_pos).scale(0.1);
        }

        let speed = player.speed.clone();
        player.pos += speed.scale((timestamp - self.last_tick) as f32 * 0.05);
        self.last_tick = timestamp;
        self.renderables.clone()
    }
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    let player = Player {
        pos: na::Vector2::new(100.0, 100.0),
        speed: na::Vector2::zeros(),
    };
    engine::start(Box::new(SomeWorld::new(player)) as Box<dyn World>);
}
