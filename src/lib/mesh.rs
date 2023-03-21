//Datastructure to define a mesh

use crate::*;
use bytemuck::*;
use ogl33::*;

pub type VertIndicies = [u32; 3];

#[derive(Copy, Clone)]
pub struct Vertex(pub [f32;3], pub [f32;3], pub [f32;2]); //Position, Normal, TextureCoords

impl Default for Vertex {
    fn default() -> Self {
        Self([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0])
    }
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<VertIndicies>,
    pub vao: VertexArray,
    pub vbo: Buffer,
    pub ebo: Buffer,
}

impl Mesh {
    pub fn new(v: Vec<Vertex>, i: Vec<VertIndicies>) -> Self {
        Self {
            vertices: v,
            indices: i,
            vao: VertexArray::new().expect("Failed to create vertex array"),
            vbo: Buffer::new().expect("Failed to create vertex buffer"),
            ebo: Buffer::new().expect("Failed to create element buffer"),
        }
    }

    pub unsafe fn setup(&mut self) {
        self.vao = VertexArray::new().expect("Failed to create vertex array");
        self.vao.bind();

        self.vbo = Buffer::new().expect("Failed to create vertex buffer");
        self.vbo.bind(GL_ARRAY_BUFFER);
        self.vbo.set_data(
            GL_ARRAY_BUFFER,
            cast_slice(self.vertices.as_slice()),
            GL_STATIC_DRAW,
        );

        self.ebo = Buffer::new().expect("Failed to create element buffer");
        self.ebo.bind(GL_ELEMENT_ARRAY_BUFFER);
        self.ebo.set_data(
            GL_ELEMENT_ARRAY_BUFFER,
            cast_slice(self.indices.as_slice()),
            GL_STATIC_DRAW,
        );

        glEnableVertexAttribArray(0);
        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::ptr::null(),
        );

        glEnableVertexAttribArray(1);
        glVertexAttribPointer(
            1,
            3,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            (3 * std::mem::size_of::<f32>()) as *const _,
        );

        glEnableVertexAttribArray(2);
        glVertexAttribPointer(
            2,
            2,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            (6 * std::mem::size_of::<f32>()) as *const _,
        );
    }
}
