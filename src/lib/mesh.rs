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
    pub vao: Option<VertexArray>,
    pub vbo: Option<Buffer>,
    pub ebo: Option<Buffer>,
}

impl Mesh {
    pub fn new(v: Vec<Vertex>, i: Vec<VertIndicies>) -> Self {
        Self {
            vertices: v,
            indices: i,
            vao: None,
            vbo: None,
            ebo: None,
        }
    }

    pub unsafe fn setup(&mut self) -> &Self {
        self.vao = Some(VertexArray::new().expect("Failed to create vertex array"));
        self.vao.unwrap().bind();

        self.vbo = Some(Buffer::new().expect("Failed to create vertex buffer"));
        self.vbo.unwrap().bind(GL_ARRAY_BUFFER);
        self.vbo.unwrap().set_data(
            GL_ARRAY_BUFFER,
            cast_slice(self.vertices.as_slice()),
            GL_STATIC_DRAW,
        );

        self.ebo = Some(Buffer::new().expect("Failed to create element buffer"));
        self.ebo.unwrap().bind(GL_ELEMENT_ARRAY_BUFFER);
        self.ebo.unwrap().set_data(
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

        //glEnableVertexAttribArray(2);
        // glVertexAttribPointer(
        //     2,
        //     2,
        //     GL_FLOAT,
        //     GL_FALSE,
        //     std::mem::size_of::<Vertex>().try_into().unwrap(),
        //     (6 * std::mem::size_of::<f32>()) as *const _,
        // );

        self
    }
}
