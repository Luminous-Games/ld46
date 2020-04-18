extern crate nalgebra as na;
const FLOAT32_BYTES: i32 = 4;

use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};

const MAX_QUADS: usize = 2;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;
const VERTEX_SIZE: usize = 5;

pub struct Renderer {
    vertices: Vec<f32>, //Box<[f32; MAX_VERTICES * VERTEX_SIZE]>,
    indices: Vec<u16>,  //Box<[i32; MAX_INDICES]>,
    num_quads: usize,

    gl: WebGlRenderingContext,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
}

impl Renderer {
    pub fn new(gl: WebGlRenderingContext, program: WebGlProgram) -> Renderer {
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
            indices: indices,
            vertices: Vec::with_capacity(MAX_VERTICES * VERTEX_SIZE),
            num_quads: 0,

            gl: gl,
            program: program,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        }
    }

    pub fn draw_quad(&mut self, pos: na::Vector2<f32>, size: na::Vector2<f32>) {
        self.vertices.push(pos.x - size.x);
        self.vertices.push(pos.y);
        self.vertices.push(1.0);
        self.vertices.push(0.0);
        self.vertices.push(0.0);

        self.vertices.push(pos.x + size.x);
        self.vertices.push(pos.y);
        self.vertices.push(1.0);
        self.vertices.push(0.0);
        self.vertices.push(0.0);

        self.vertices.push(pos.x - size.x);
        self.vertices.push(pos.y + size.y);
        self.vertices.push(1.0);
        self.vertices.push(0.0);
        self.vertices.push(0.0);

        self.vertices.push(pos.x + size.x);
        self.vertices.push(pos.y + size.y);
        self.vertices.push(1.0);
        self.vertices.push(0.0);
        self.vertices.push(0.0);

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
            self.gl.get_attrib_location(&self.program, "position") as u32;
        let color_attrib_location = self.gl.get_attrib_location(&self.program, "color") as u32;

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

        self.gl.enable_vertex_attrib_array(position_attrib_location);
        self.gl.enable_vertex_attrib_array(color_attrib_location);

        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            (self.num_quads as i32) * 6,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );

        self.num_quads = 0;
        self.vertices.clear();
    }
}
