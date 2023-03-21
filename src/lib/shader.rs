//Builder pattern for shader program

use std::ffi::CString;

use ogl33::*;


pub enum ShaderType {
    Vertex=GL_VERTEX_SHADER as isize,
    Fragment=GL_FRAGMENT_SHADER as isize,
}


pub struct ShaderProgramBuilder {
    id: GLuint,
}

pub struct ShaderProgram(pub GLuint);

impl ShaderProgramBuilder {
    pub fn new() -> Self {
        unsafe{
            Self {
                id: glCreateProgram(),
            }
        }
    }

    pub fn attach_shader(&self, shader_type: ShaderType, shader_src: &str) -> &Self {
        unsafe {
            let shader = compile_shader(shader_type, shader_src).expect("Failed to compile shader");
            glAttachShader(self.id, shader);
            glDeleteShader(shader);
        }

        self
    }

    pub fn link(&self) -> Option<ShaderProgram> {
        unsafe {
            glLinkProgram(self.id);

            let mut success = 0;
            glGetProgramiv(self.id, GL_LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;

                glGetProgramInfoLog(self.id, 1024, &mut log_len, v.as_mut_ptr().cast());
                v.set_len(log_len.try_into().unwrap());
                panic!(
                    "Shader program linking failed: {}",
                    String::from_utf8(v).unwrap()
                );
            }
            
            Some(ShaderProgram(self.id))
        }
    }
}


unsafe fn compile_shader(shader_type: ShaderType, shader_src: &str) -> Option<u32> {
    
    let shader = glCreateShader(shader_type as u32);
    let shader_src = CString::new(shader_src).unwrap();
    glShaderSource(shader, 1, &shader_src.as_ptr(), std::ptr::null());

    glCompileShader(shader);

    let mut success = 0;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut v: Vec<u8> = Vec::with_capacity(1024);
        let mut log_len = 0_i32;

        glGetShaderInfoLog(shader, 1024, &mut log_len, v.as_mut_ptr().cast());
        v.set_len(log_len.try_into().unwrap());
        panic!(
            "Shader compilation failed: {}",
            String::from_utf8(v).unwrap()
        );
    }

    Some(shader)
}
