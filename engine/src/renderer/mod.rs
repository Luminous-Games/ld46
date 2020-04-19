extern crate nalgebra as na;

use std::collections::HashMap;

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlTexture};

const FLOAT32_BYTES: i32 = 4;

const MAX_QUADS: usize = 10000;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;
const VERTEX_SIZE: usize = 8;

mod glutil;

#[derive(Clone)]
pub struct TextureMap {
    tiles_x: i32,
    tiles_y: i32,
}

impl TextureMap {
    pub fn new(tiles_x: i32, tiles_y: i32) -> TextureMap {
        TextureMap {
            tiles_x: tiles_x,
            tiles_y: tiles_y,
        }
    }
    pub fn get_texture(&self, column: i32, row: i32) -> Texture {
        let width = 1f32 / self.tiles_x as f32;
        let height = 1f32 / self.tiles_y as f32;

        return Texture {
            start: na::Vector2::new(width * column as f32, height * row as f32),
            size: na::Vector2::new(width, height),
        };
    }

    pub fn get_texture_custom(&self, column: f32, row: f32, w: f32, h: f32) -> Texture {
        let width = 1f32 / self.tiles_x as f32;
        let height = 1f32 / self.tiles_y as f32;

        Texture {
            start: na::Vector2::new(width * column as f32, height * row as f32),
            size: na::Vector2::new(width * w, height * h),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    start: na::Vector2<f32>,
    size: na::Vector2<f32>,
}

pub struct Renderer {
    vertices: Vec<f32>, //Box<[f32; MAX_VERTICES * VERTEX_SIZE]>,
    indices: Vec<u16>,  //Box<[i32; MAX_INDICES]>,
    num_quads: usize,

    pub gl: WebGlRenderingContext,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    texture: WebGlTexture,

    programs: HashMap<i32, WebGlProgram>,
    selected_program: i32,
    viewport: na::Vector2<f32>,
    camera: na::Point2<f32>,
}

impl Renderer {
    pub fn new(gl: WebGlRenderingContext, texture: WebGlTexture) -> Renderer {
        // Configure GL
        gl.enable(WebGlRenderingContext::DEPTH_TEST);
        gl.depth_func(WebGlRenderingContext::LESS);

        gl.enable(WebGlRenderingContext::BLEND);
        gl.blend_func(
            WebGlRenderingContext::ONE,
            WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // Initialise indices
        let mut indices = Vec::with_capacity(MAX_INDICES);
        let mut offset = 0;
        for _ in 0..MAX_QUADS {
            indices.push(offset + 0);
            indices.push(offset + 1);
            indices.push(offset + 2);

            indices.push(offset + 2);
            indices.push(offset + 3);
            indices.push(offset + 1);

            offset += 4;
        }

        // Initialise buffers
        let vertex_buffer = gl
            .create_buffer()
            .ok_or("failed to create vertex buffer")
            .unwrap();

        let index_buffer = gl
            .create_buffer()
            .ok_or("failed to create index buffer")
            .unwrap();

        Renderer {
            indices,
            vertices: Vec::with_capacity(MAX_VERTICES * VERTEX_SIZE),
            num_quads: 0,

            gl,
            vertex_buffer,
            index_buffer,
            texture,

            selected_program: 0,
            programs: HashMap::new(),
            viewport: na::Vector2::zeros(),
            camera: na::Point2::new(0.0, 0.0),
        }
    }

    pub fn load_shader(&mut self, vertex: &str, fragment: &str) -> i32 {
        let vert_shader =
            glutil::compile_shader(&self.gl, WebGlRenderingContext::VERTEX_SHADER, vertex).unwrap();
        let frag_shader =
            glutil::compile_shader(&self.gl, WebGlRenderingContext::FRAGMENT_SHADER, fragment)
                .unwrap();

        let program = glutil::link_program(&self.gl, &vert_shader, &frag_shader).unwrap();
        let key = self.programs.len() as i32;
        self.programs.insert(key, program);

        return key;
    }

    pub fn set_viewport(&mut self, viewport: na::Vector2<f32>) {
        self.viewport = viewport;
    }

    pub fn set_camera(&mut self, camera: na::Point2<f32>) {
        self.camera = camera;
    }

    pub fn draw_quad(&mut self, pos: na::Point2<f32>, size: na::Vector2<f32>, texture: &Texture) {
        self.vertices.push(pos.x - size.x / 2.0);
        self.vertices.push(pos.y);
        self.vertices.push(-pos.y); //depth
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x);
        self.vertices.push(texture.start.y + texture.size.y);

        self.vertices.push(pos.x + size.x / 2.0);
        self.vertices.push(pos.y);
        self.vertices.push(-pos.y); //depth
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x + texture.size.x);
        self.vertices.push(texture.start.y + texture.size.y);

        self.vertices.push(pos.x - size.x / 2.0);
        self.vertices.push(pos.y + size.y);
        self.vertices.push(-pos.y); //depth
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x);
        self.vertices.push(texture.start.y);

        self.vertices.push(pos.x + size.x / 2.0);
        self.vertices.push(pos.y + size.y);
        self.vertices.push(-pos.y); //depth
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x + texture.size.x);
        self.vertices.push(texture.start.y);

        self.num_quads += 1;
    }

    pub fn flush(&mut self) {
        let program = self.programs.get(&self.selected_program).unwrap();
        self.gl.use_program(Some(program));
        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.vertex_buffer),
        );

        // danger zone: this is a live view to memory. No allocations in this block!
        unsafe {
            let vert_array = js_sys::Float32Array::view(&self.vertices.as_slice());

            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        self.gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.index_buffer),
        );

        // danger zone: this is a live view to memory. No allocations in this block!
        unsafe {
            let index_array = js_sys::Uint16Array::view(&self.indices.as_slice());

            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &index_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let position_attrib_location = self.gl.get_attrib_location(&program, "aPosition") as u32;
        let color_attrib_location = self.gl.get_attrib_location(&program, "aColor") as u32;
        let texcoord_attrib_location = self.gl.get_attrib_location(&program, "aTexCoord") as u32;

        let sampler_uniform_location = self.gl.get_uniform_location(&program, "uSampler").unwrap();

        let viewport_uniform_location =
            self.gl.get_uniform_location(&program, "uViewport").unwrap();
        let viewport_transform_location = self
            .gl
            .get_uniform_location(&program, "uTransform")
            .unwrap();

        self.gl.vertex_attrib_pointer_with_i32(
            position_attrib_location,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            (VERTEX_SIZE as i32) * FLOAT32_BYTES,
            0,
        );
        self.gl.vertex_attrib_pointer_with_i32(
            color_attrib_location,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            (VERTEX_SIZE as i32) * FLOAT32_BYTES,
            3 * FLOAT32_BYTES,
        );
        self.gl.vertex_attrib_pointer_with_i32(
            texcoord_attrib_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            (VERTEX_SIZE as i32) * FLOAT32_BYTES,
            6 * FLOAT32_BYTES,
        );

        let camera_pos_transform =
            na::Translation3::new(-self.camera.x, -self.camera.y, -self.camera.y);
        let orthographic_view = na::Orthographic3::new(
            -self.viewport.x / 2.0,
            self.viewport.x / 2.0,
            -self.viewport.y / 2.0,
            self.viewport.y / 2.0,
            -self.viewport.y * 2.0 + self.camera.y,
            self.viewport.y + self.camera.y,
        );

        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&viewport_uniform_location),
            false,
            orthographic_view.as_matrix().as_slice(),
        );

        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&viewport_transform_location),
            false,
            camera_pos_transform.to_homogeneous().as_slice(),
        );

        // Use texture 0

        self.gl.enable_vertex_attrib_array(position_attrib_location);
        self.gl.enable_vertex_attrib_array(color_attrib_location);
        self.gl.enable_vertex_attrib_array(texcoord_attrib_location);

        self.gl.active_texture(WebGlRenderingContext::TEXTURE0);
        self.gl
            .bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));
        self.gl.uniform1i(Some(&sampler_uniform_location), 0);

        self.gl.clear_color(0.4, 0.57, 0.4, 1.0);
        self.gl.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        self.gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            (self.num_quads as i32) * 6,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );

        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.color_mask(false, false, false, true);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        self.gl.color_mask(true, true, true, true);

        self.num_quads = 0;
        self.vertices.clear();
    }
}
