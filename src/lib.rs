use __core::{ops::Deref};
use bytemuck::*;
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3};
use ogl33::*;
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
};

//Wrapper for opengl buffer objects
#[derive(Clone, Copy)]
pub struct Buffer(pub GLuint, pub GLenum, pub GLenum); //Buffer, Target, Usage

impl Buffer {
    pub unsafe fn new(usage: GLenum, target: GLenum) -> Option<Self> {
        let mut buffer = 0;
        glGenBuffers(1, &mut buffer);

        if buffer == 0 {
            None
        } else {
            Some(Self(buffer, target, usage))
        }
    }

    pub unsafe fn bind(&self) -> &Self {
        glBindBuffer(self.1, self.0);
        self
    }

    pub unsafe fn unbind(&self) -> &Self {
        glBindBuffer(self.1, 0);
        self
    }

    pub unsafe fn set_data(&self, data: &[u8]) {
        glBufferData(
            self.1,
            data.len().try_into().unwrap(),
            data.as_ptr() as *const _,
            self.2,
        );
    }
}

//Buffer for opengl vertex array objects
#[derive(Clone, Copy)]
pub struct VertexArray(pub GLuint);

impl VertexArray {
    pub unsafe fn new() -> Option<Self> {
        let mut vao = 0;

        glGenVertexArrays(1, &mut vao);

        if vao == 0 {
            None
        } else {
            Some(Self(vao))
        }
    }

    pub unsafe fn unbind() {
        glBindVertexArray(0);
    }

    pub unsafe fn bind(&self) {
        glBindVertexArray(self.0);
    }
}

pub enum ShaderType {
    Vertex = GL_VERTEX_SHADER as isize,
    Fragment = GL_FRAGMENT_SHADER as isize,
}

pub struct ShaderProgramBuilder {
    id: GLuint,
    uniforms: HashMap<String, GLint>,
}

//Wrapper for opengl shader programs (uses builder pattern)
pub struct ShaderProgram(pub GLuint, pub HashMap<String, GLint>); //Program, Uniforms

impl ShaderProgram {
    pub unsafe fn create_uniform(&mut self, name: &CStr) {
        let uniform_location = glGetUniformLocation(self.0, name.as_ptr());
        if uniform_location < 0 {
            panic!("Failed to find uniform {}", name.to_str().unwrap());
        } else {
            self.1
                .insert(name.to_str().unwrap().to_string(), uniform_location);
        }
    }
    pub unsafe fn set_mat4(&self, name: &str, mat: &Matrix4<f32>) {
        glUniformMatrix4fv(
            self.1[name],
            1,
            GL_FALSE,
            mat as *const Matrix4<f32> as *const f32,
        );
    }

    pub unsafe fn set_vec3(&self, name: &str, vec: &cgmath::Vector3<f32>) {
        glUniform3f(self.1[name], vec.x, vec.y, vec.z);
    }

    pub unsafe fn set_vec4(&self, name: &str, vec: &cgmath::Vector4<f32>) {
        glUniform4f(self.1[name], vec.x, vec.y, vec.z, vec.w);
    }
}

impl ShaderProgramBuilder {
    pub fn new() -> Self {
        unsafe {
            Self {
                id: glCreateProgram(),
                uniforms: HashMap::new(),
            }
        }
    }
    //Compiles shader from source and attaches it to the program, as-well as deleting it after attachment
    pub fn create_shader(&self, shader_type: ShaderType, shader_src: &str) -> &Self {
        unsafe {
            let shader = compile_shader(shader_type, shader_src).expect("Failed to compile shader");
            glAttachShader(self.id, shader);
            glDeleteShader(shader);
        }

        self
    }
    //Links the program and returns a ShaderProgram wrapper
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

            Some(ShaderProgram(self.id, self.uniforms.clone()))
        }
    }
}

//Compiles a shader from a string source and returns the shader id
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

//Parse an obj file and return a mesh (HEAVILY WIP)
pub fn mesh_from_obj(path: &Path) -> Mesh {
    let (models,_) = tobj::load_obj(path, &tobj::LoadOptions::default()).unwrap();



    Mesh{
        vertices:Vec::new(),
        indicies:Vec::new(),
        vao: None,
        vbo: None,
        ebo: None,
    }
}

pub type VertIndicies = [u32; 3];

pub type Vertex=[f32; 3+2]; //Position, Normal, TextureCoords

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indicies: Vec<VertIndicies>,
    pub vao: Option<VertexArray>,
    pub vbo: Option<Buffer>,
    pub ebo: Option<Buffer>,
}

impl Mesh {
    pub fn new(v: Vec<Vertex>, i: Vec<VertIndicies>) -> Self {
        Self {
            vertices: v,
            indicies: i,
            vao: None,
            vbo: None,
            ebo: None,
        }
    }

    pub unsafe fn setup(&mut self) -> &Self {
        self.vao = Some(VertexArray::new().expect("Failed to create vertex array"));
        self.vao.unwrap().bind();

        self.vbo = Some(
            Buffer::new(GL_STATIC_DRAW, GL_ARRAY_BUFFER).expect("Failed to create vertex buffer"),
        );
        self.vbo.unwrap().bind();
        self.vbo
            .unwrap()
            .set_data(cast_slice(self.vertices.as_slice()));

        self.ebo = Some(
            Buffer::new(GL_STATIC_DRAW, GL_ELEMENT_ARRAY_BUFFER)
                .expect("Failed to create element buffer"),
        );
        self.ebo.unwrap().bind();
        self.ebo
            .unwrap()
            .set_data(cast_slice(self.indicies.as_slice()));

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
            2,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 6]>() as *const _,
        );

        self
    }
}

pub struct Camera {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub fov: f32,
    pub aspect: [f32; 2],
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(start_pos: Vector3<f32>, targ: Vector3<f32>) -> Self {
        Self {
            position: start_pos,
            target: targ,
            fov: 45.0,
            aspect: [4.0, 3.0],
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }

    pub fn get_right(&self) -> Vector3<f32> {
        self.get_direction()
            .cross(Vector3::new(0.0, 1.0, 0.0))
            .normalize()
    }

    pub fn get_up(&self) -> Vector3<f32> {
        self.get_right().cross(self.get_direction()).normalize()
    }

    pub fn set_position(&mut self, pos: Vector3<f32>) {
        self.position = pos;
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let pos_p = Point3::new(self.position.x, self.position.y, self.position.z);
        let target_p = Point3::new(self.target.x, self.target.y, self.target.z);
        let up_v = self.get_up();
        let up_p = Vector3::new(up_v.x, up_v.y, up_v.z);
        Matrix4::look_at_rh(pos_p, target_p, up_p)
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        cgmath::PerspectiveFov {
            fovy: Deg(self.fov).into(),
            aspect: self.aspect[0] / self.aspect[1],
            near: self.near,
            far: self.far,
        }
        .into()
    }
}
