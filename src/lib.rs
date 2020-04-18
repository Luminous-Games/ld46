#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use engine::key::key_codes;
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
}

struct SomeBox {}

impl Renderable for SomeBox {
    fn render(&self, r: &mut engine::renderer::Renderer) {
        let tm = engine::renderer::TextureMap::new(4, 4);

        r.draw_quad(
            na::Vector2::new(300.0, 100.0),
            na::Vector2::new(200.0, 200.0),
            tm.get_texture(3, 2),
        );
    }
}

#[derive(Clone)]
struct Player {
    x: f32,
    y: f32,
}

impl Renderable for Player {
    fn render(&self, r: &mut Renderer) {
        let tm = engine::renderer::TextureMap::new(4, 4);
        r.draw_quad(
            na::Vector2::new(self.x, self.y),
            na::Vector2::new(0.3, 0.1),
            tm.get_texture(3, 2),
        );
    }
}

impl SomeWorld {
    fn new(player: Player) -> SomeWorld {
        let player_in_a_box = Rc::new(RefCell::new(player));
        SomeWorld {
            renderables: vec![player_in_a_box.clone()],
            player: player_in_a_box.clone(),
        }
    }
}

impl engine::World for SomeWorld {
    fn tick<'a>(
        &'a self,
        key_manager: &engine::key::KeyManager,
    ) -> Vec<Rc<RefCell<dyn Renderable>>> {
        let mut player = self.player.borrow_mut();
        if (key_manager.key_pressed(key_codes::W)) {
            player.y += 0.01;
        }
        if (key_manager.key_pressed(key_codes::S)) {
            player.y -= 0.01;
        }
        if (key_manager.key_pressed(key_codes::D)) {
            player.x += 0.01;
        }
        if (key_manager.key_pressed(key_codes::A)) {
            player.x -= 0.01;
        }
        self.renderables.clone()
    }
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    let player = Player { x: 0.4, y: 0.4 };
    engine::start(Box::new(SomeWorld::new(player)) as Box<dyn World>);
}
