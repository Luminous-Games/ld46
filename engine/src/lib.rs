use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

extern crate nalgebra as na;

pub mod renderer;
use renderer::Renderer;

pub trait Renderable {
    fn render(&self, renderer: &mut Renderer);
}

pub trait World {
    fn tick<'a>(&'a self) -> &'a Vec<Box<dyn Renderable>>;
}
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn start(world: Box<dyn World>) -> Result<(), JsValue> {
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
    gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);

    let vert_shader = compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("shaders/vertex.glsl"),
    )
    .unwrap();
    let frag_shader = compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        include_str!("shaders/fragment.glsl"),
    )
    .unwrap();

    let program = link_program(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    let mut renderer = Renderer::new(gl, program, texture);
    log! {"Engine initialised"};

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let viewport = na::Vector2::new(canvas.width() as f32, canvas.height() as f32);
        log!("{:?}", viewport);
        renderer
            .gl
            .viewport(0, 0, viewport.x as i32, viewport.y as i32);

        renderer.set_viewport(viewport);

        let renderables = world.tick();
        for renderable in renderables {
            renderable.render(&mut renderer);
        }

        renderer.flush();

        request_animation_frame(f.borrow().as_ref().unwrap());
        // let _ = f.borrow_mut().take();
        // return;
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        gl.validate_program(&program);
        if gl
            .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(gl
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error validating program object")))
        }
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
