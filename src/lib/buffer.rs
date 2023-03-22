//Creates various opengl buffers

use ogl33::*;


#[derive(Clone,Copy)]
pub struct Buffer(pub GLuint);

impl Buffer {
    pub fn new() -> Option<Self> {
        let mut buffer = 0;
        unsafe {
            glGenBuffers(1, &mut buffer);
        }
        if buffer == 0 {
            None
        } else {
            Some(Self(buffer))
        }
    }

    pub fn bind(&self, target: GLenum) -> &Self {
        unsafe {
            glBindBuffer(target, self.0);
        }
        self
    }

    pub fn set_data(&self, target: GLenum, data: &[u8], usage: GLenum) {
        unsafe {
            glBufferData(
                target,
                data.len().try_into().unwrap(),
                data.as_ptr() as *const _,
                usage,
            );
        }
    }
}
