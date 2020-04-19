extern crate nalgebra as na;
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;

use renderer::Renderer;

pub mod key;
pub mod renderer;

pub trait Renderable {
    fn render(&self, renderer: &mut Renderer);
}

pub trait World {
    fn tick<'a>(
        &'a mut self,
        key_manager: &key::KeyManager,
        timestamp: f64,
    ) -> Vec<Rc<RefCell<dyn Renderable>>>;
}
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn start(mut world: Box<dyn World>) -> Result<(), JsValue> {
    let mut key_manager = key::KeyManager::new();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let texture_image = document
        .get_element_by_id("texture")
        .unwrap()
        .dyn_into::<web_sys::HtmlImageElement>()?;

    let gl = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let texture = gl.create_texture().unwrap();

    gl.active_texture(WebGlRenderingContext::TEXTURE0);
    gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

    gl.tex_image_2d_with_u32_and_u32_and_image(
        WebGlRenderingContext::TEXTURE_2D,
        0,
        WebGlRenderingContext::RGBA as i32,
        WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,
        &texture_image,
    )
    .unwrap();

    gl.tex_parameteri(
        WebGlRenderingContext::TEXTURE_2D,
        WebGlRenderingContext::TEXTURE_MIN_FILTER,
        WebGlRenderingContext::NEAREST as i32,
    );
    gl.tex_parameteri(
        WebGlRenderingContext::TEXTURE_2D,
        WebGlRenderingContext::TEXTURE_MAG_FILTER,
        WebGlRenderingContext::NEAREST as i32,
    );

    gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);

    let mut renderer = Renderer::new(gl, texture);
    renderer.load_shader(
        include_str!("shaders/vertex.glsl"),
        include_str!("shaders/fragment.glsl"),
    );
    log! {"Engine initialised"};

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let viewport = na::Vector2::new(canvas.width() as f32, canvas.height() as f32);
        log!("{:?}: {}", viewport, timestamp);
        renderer
            .gl
            .viewport(0, 0, viewport.x as i32, viewport.y as i32);

        renderer.set_viewport(viewport);

        let renderables = world.tick(&key_manager, timestamp);
        key_manager.post_tick_update_key_states();
        for renderable in renderables {
            renderable.borrow().render(&mut renderer);
        }

        renderer.flush();

        request_animation_frame(f.borrow().as_ref().unwrap());
        // let _ = f.borrow_mut().take();
        // return;
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
