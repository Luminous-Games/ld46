#[cfg(debug_assertions)]
extern crate console_error_panic_hook;
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate nalgebra as na;
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use wasm_bindgen::prelude::*;

struct SomeWorld {
    renderables: Vec<Box<dyn engine::Renderable>>,
}
struct SomeBox {}

impl engine::Renderable for SomeBox {
    fn render(&self, r: &mut engine::renderer::Renderer) {
        let tm = engine::renderer::TextureMap::new(4, 4);

        r.draw_quad(
            na::Vector2::new(300.0, 100.0),
            na::Vector2::new(200.0, 200.0),
            tm.get_texture(3, 2),
        );
    }
}

impl engine::World for SomeWorld {
    fn tick(&self) -> &Vec<Box<dyn engine::Renderable>> {
        return &self.renderables;
    }
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    engine::start(Box::new(SomeWorld {
        renderables: vec![Box::new(SomeBox {})],
    }))
    .unwrap();
}
