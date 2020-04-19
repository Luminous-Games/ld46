extern crate nalgebra as na;
#[macro_use]
extern crate num_derive;
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;

use renderer::Renderer;

pub mod key;
pub mod mouse;
pub mod renderer;

pub trait Renderable {
    fn render(&self, renderer: &mut Renderer);
}

pub trait World {
    fn tick(&mut self, key_manager: &key::KeyManager, timestamp: f64);
    fn get_game_objects(&self) -> Vec<&GameObject>;
}
pub struct GameObject {
    pub pos: na::Point2<f32>,
    pub speed: na::Vector2<f32>,
    collider: Option<Collider>,
    rend: Vec<Box<dyn Rend>>,
}
impl GameObject {
    pub fn new(pos: na::Point2<f32>) -> GameObject {
        GameObject {
            pos,
            speed: na::Vector2::zeros(),
            collider: None,
            rend: vec![],
        }
    }

    pub fn add_collider(&mut self, collider: Collider) {
        self.collider = Some(collider);
    }

    pub fn add_rend(&mut self, rend: Box<dyn Rend>) {
        self.rend.push(rend);
    }

    pub fn get_collider(&self) -> &Option<Collider> {
        &self.collider
    }
}

pub trait Rend {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject);
}

pub struct Collider {
    range: f32,
}

impl Collider {
    pub fn new(range: f32) -> Collider {
        Collider { range }
    }

    fn get_range(&self) -> f32 {
        return self.range;
    }

    pub fn collide(
        &self,
        game_object: &GameObject,
        pos: &na::Point2<f32>,
        speed: &na::Vector2<f32>,
    ) -> na::Vector2<f32> {
        let mut speed = speed.clone_owned();
        if na::distance_squared(&game_object.pos, pos) < (self.get_range() + 16.0).powi(2) {
            if pos.x > game_object.pos.x {
                speed.x = f32::max(0.0, speed.x);
            } else {
                speed.x = f32::min(0.0, speed.x);
            }
            if pos.y > game_object.pos.y {
                speed.y = f32::max(0.0, speed.y);
            } else {
                speed.y = f32::min(0.0, speed.y);
            }
        };
        speed
    }
}

impl Renderable for GameObject {
    fn render(&self, renderer: &mut Renderer) {
        for r in self.rend.iter() {
            r.render(renderer, &self);
        }
    }
}

pub fn start(mut world: Box<dyn World>) -> Result<(), JsValue> {
    let mut key_manager = key::KeyManager::new();
    let mut mouse_manager = mouse::MouseManager::new();

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
    log::info! {"Engine initialised"};

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let viewport = na::Vector2::new(canvas.width() as f32, canvas.height() as f32);
        log::debug!("{:?}: {}", viewport, timestamp);
        renderer
            .gl
            .viewport(0, 0, viewport.x as i32, viewport.y as i32);

        renderer.set_viewport(viewport);

        mouse_manager.pre_tick_process_mouse_state();
        world.tick(&key_manager, timestamp);
        let gameobjects = world.get_game_objects();
        key_manager.post_tick_update_key_states();
        for gameobject in gameobjects.iter() {
            gameobject.render(&mut renderer);
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
