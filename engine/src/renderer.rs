extern crate nalgebra as na;
const FLOAT32_BYTES: i32 = 4;

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlTexture};

const MAX_QUADS: usize = 2;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;
const VERTEX_SIZE: usize = 7;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

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
}

#[derive(Debug)]
pub struct Texture {
    start: na::Vector2<f32>,
    size: na::Vector2<f32>,
}

pub struct Renderer {
    vertices: Vec<f32>, //Box<[f32; MAX_VERTICES * VERTEX_SIZE]>,
    indices: Vec<u16>,  //Box<[i32; MAX_INDICES]>,
    num_quads: usize,

    pub gl: WebGlRenderingContext,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    texture: WebGlTexture,

    viewport: na::Vector2<f32>,
}

impl Renderer {
    pub fn new(
        gl: WebGlRenderingContext,
        program: WebGlProgram,
        texture: WebGlTexture,
    ) -> Renderer {
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
            program,
            vertex_buffer,
            index_buffer,
            texture,
            viewport: na::Vector2::zeros(),
        }
    }

    pub fn set_viewport(&mut self, viewport: na::Vector2<f32>) {
        self.viewport = viewport;
    }

    pub fn draw_quad(&mut self, pos: na::Vector2<f32>, size: na::Vector2<f32>, texture: Texture) {
        self.vertices.push(pos.x - size.x / 2.0);
        self.vertices.push(pos.y);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x);
        self.vertices.push(texture.start.y + texture.size.y);

        self.vertices.push(pos.x + size.x / 2.0);
        self.vertices.push(pos.y);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x + texture.size.x);
        self.vertices.push(texture.start.y + texture.size.y);

        self.vertices.push(pos.x - size.x / 2.0);
        self.vertices.push(pos.y + size.y);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x);
        self.vertices.push(texture.start.y);

        self.vertices.push(pos.x + size.x / 2.0);
        self.vertices.push(pos.y + size.y);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(1.0);
        self.vertices.push(texture.start.x + texture.size.x);
        self.vertices.push(texture.start.y);

        self.num_quads += 1;
    }

    pub fn flush(&mut self) {
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

        let position_attrib_location =
            self.gl.get_attrib_location(&self.program, "aPosition") as u32;
        let color_attrib_location = self.gl.get_attrib_location(&self.program, "aColor") as u32;
        let texcoord_attrib_location =
            self.gl.get_attrib_location(&self.program, "aTexCoord") as u32;

        let sampler_uniform_location = self
            .gl
            .get_uniform_location(&self.program, "uSampler")
            .unwrap();

        let viewport_uniform_location = self
            .gl
            .get_uniform_location(&self.program, "uViewport")
            .unwrap();

        self.gl.vertex_attrib_pointer_with_i32(
            position_attrib_location,
            2,
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
            2 * FLOAT32_BYTES,
        );
        self.gl.vertex_attrib_pointer_with_i32(
            texcoord_attrib_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            (VERTEX_SIZE as i32) * FLOAT32_BYTES,
            5 * FLOAT32_BYTES,
        );

        let orthographic_view =
            na::Orthographic3::new(0.0, self.viewport.x, 0.0, self.viewport.y, 0.0, 1.0);

        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&viewport_uniform_location),
            false,
            orthographic_view.as_matrix().as_slice(),
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
