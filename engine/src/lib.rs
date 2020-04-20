#[macro_use]
extern crate downcast_rs;
extern crate nalgebra as na;
#[macro_use]
extern crate num_derive;
extern crate wee_alloc;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use downcast_rs::Downcast;
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
    pub rend: Vec<Box<dyn Rend>>,
    pub props: HashMap<String, f32>,
}
impl GameObject {
    pub fn new(pos: na::Point2<f32>) -> GameObject {
        GameObject {
            pos,
            speed: na::Vector2::zeros(),
            collider: None,
            rend: vec![],
            props: HashMap::new(),
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

pub trait Rend: Downcast {
    fn render(&self, renderer: &mut Renderer, game_object: &GameObject);
}
impl_downcast!(Rend);

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
        speed: &mut na::Vector2<f32>,
    ) -> bool {
        if f32::abs(game_object.pos.x - pos.x) > (self.get_range() + 16.0)
            || f32::abs(game_object.pos.y - pos.y) > (self.get_range() + 16.0)
        {
            return false;
        }
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
            return true;
        } else {
            return false;
        };
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

    let gl = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let mut renderer = Renderer::new(gl);
    renderer.load_shader(
        include_str!("shaders/vertex.glsl"),
        include_str!("shaders/fragment.glsl"),
    );
    log::info! {"Engine initialised"};

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let viewport = na::Vector2::new(canvas.width() as f32, canvas.height() as f32);
        // log::debug!("{:?}: {}", viewport, timestamp);
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
