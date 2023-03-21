//Creates various opengl buffers

use ogl33::*;



pub struct Buffer(pub GLuint);

impl Buffer{
    pub fn new() -> Option<Self>{
        let mut buffer = 0;
        unsafe{
            glGenBuffers(1, &mut buffer);
        }
        if buffer == 0 {
            None
        }
        else{ 
            Some(Self(buffer))
        }
    }

    pub fn bind(&self, target: GLenum){
        unsafe{
            glBindBuffer(target, self.0);
        }
    }
}