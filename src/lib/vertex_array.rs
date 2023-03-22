use ogl33::*;


#[derive(Clone,Copy)]
pub struct VertexArray(pub GLuint);

impl VertexArray {
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe {
            glGenVertexArrays(1, &mut vao);
        }
        if vao == 0 {
            None
        } else {
            Some(Self(vao))
        }
    }

    pub fn bind(&self) {
        unsafe {
            glBindVertexArray(self.0);
        }
    }
}
