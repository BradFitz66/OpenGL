use bytemuck::*;
use cgmath::{
    Deg, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector2, Vector3,
};
use ogl33::*;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
};
use std::{fs::File, io::prelude::*, path::Path};
use radiant::*;
//Wrapper for opengl textures
#[derive(Clone)]
pub struct Texture2D(pub GLuint);

impl Texture2D {
    pub unsafe fn new(texture_unit: GLenum) -> Option<Self> {
        let mut texture = 0;
        glGenTextures(1, &mut texture);
        glActiveTexture(texture_unit);
        if texture == 0 {
            None
        } else {
            Some(Self(texture))
        }
    }

    pub unsafe fn bind(&self) {
        glBindTexture(GL_TEXTURE_2D, self.0);
    }

    pub unsafe fn unbind(&self) {
        glBindTexture(GL_TEXTURE_2D, 0);
    }

    pub unsafe fn set_data(
        &self,
        data: &[u8],
        width: i32,
        height: i32,
        format: GLenum,
        internal_format: GLint,
        type_: GLenum,
    ) {
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format,
            type_,
            data.as_ptr() as *const _,
        );
    }

    pub unsafe fn set_filter(&self, filter: GLenum) {
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filter as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filter as i32);
    }

    pub unsafe fn set_wrap(&self, wrap: GLenum) {
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap as i32);
    }
}

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

    pub unsafe fn unbind(&self) {
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
    uniformblocks: HashMap<String, GLuint>,
}

//Wrapper for opengl shader programs (uses builder pattern)
pub struct ShaderProgram(
    pub GLuint,
    pub HashMap<String, GLint>,
    pub HashMap<String, GLuint>,
); //Program, Uniforms, UniformBlocks

impl ShaderProgram {
    pub unsafe fn create_uniform(&mut self, name: &CStr) {
        let uniform_location = glGetUniformLocation(self.0, name.as_ptr());
        self.1
            .insert(name.to_str().unwrap().to_string(), uniform_location);
    }

    pub unsafe fn create_uniformblock(&mut self, name: &CStr) {
        let uniform_location = glGetUniformBlockIndex(self.0, name.as_ptr());
        self.2
            .insert(name.to_str().unwrap().to_string(), uniform_location);
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

    pub unsafe fn set_float(&self, name: &str, val: f32) {
        glUniform1f(self.1[name], val);
    }

    pub unsafe fn set_int(&self, name: &str, val: i32) {
        glUniform1i(self.1[name], val);
    }
}

impl ShaderProgramBuilder {
    pub fn new() -> Self {
        unsafe {
            Self {
                id: glCreateProgram(),
                uniforms: HashMap::new(),
                uniformblocks: HashMap::new(),
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

            Some(ShaderProgram(
                self.id,
                self.uniforms.clone(),
                self.uniformblocks.clone(),
            ))
        }
    }
}

pub fn shader_from_file(path: &Path) -> String {
    let mut file = File::open(path).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    contents
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

pub fn calculate_tangents(positions: Vec<[f32; 3]>, texcoords: Vec<[f32; 2]>) -> Vec<f32> {
    let mut tangents: Vec<f32> = Vec::new();

    for i in 0..positions.len() / 3 {
        let i = i * 3;
        let v0 = Vector3::new(positions[i][0], positions[i][1], positions[i][2]);
        let v1 = Vector3::new(
            positions[i + 1][0],
            positions[i + 1][1],
            positions[i + 1][2],
        );
        let v2 = Vector3::new(
            positions[i + 2][0],
            positions[i + 2][1],
            positions[i + 2][2],
        );

        let uv0 = Vector2::new(texcoords[i][0], texcoords[i][1]);
        let uv1 = Vector2::new(texcoords[i + 1][0], texcoords[i + 1][1]);
        let uv2 = Vector2::new(texcoords[i + 2][0], texcoords[i + 2][1]);

        let delta_pos1 = v1 - v0;
        let delta_pos2 = v2 - v0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        tangents.push(tangent.x);
        tangents.push(tangent.y);
        tangents.push(tangent.z);
    }
    tangents
}

//Parse an obj file and return a mesh (HEAVILY WIP)
pub fn mesh_from_obj(path: &Path) -> Mesh {
    let (models, _) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mesh = &models[0].mesh;

    //Convert normals from Vec<f32> to Vec<[f32;3]>
    let vertex_normals: Vec<[f32; 3]> = mesh
        .normals
        .chunks(3)
        .map(|chunk| TryInto::<[f32; 3]>::try_into(chunk).unwrap())
        .collect();

    let vertex_texcoords: Vec<[f32; 2]> = mesh
        .texcoords
        .chunks(2)
        .map(|chunk| TryInto::<[f32; 2]>::try_into(chunk).unwrap())
        .collect();

    let vertex_positions: Vec<[f32; 3]> = mesh
        .positions
        .chunks(3)
        .map(|chunk| TryInto::<[f32; 3]>::try_into(chunk).unwrap())
        .collect();

    //Convert vertex_positions from Vec<[f32;3]> to Vec<[f32;8]> (since type Vertex is [f32;3+3+2])
    //Range 0..3 is position, 3..6 is normal, 6..8 is texcoord
    let vertices: Vec<Vertex> = vertex_positions
        .into_iter()
        .zip(vertex_normals.into_iter())
        .zip(vertex_texcoords.into_iter())
        .map(|((chunk_pos, chunk_normal), chunk_tex)| {
            let mut result = [0.0; 14];
            result[0..3].copy_from_slice(&chunk_pos);
            result[3..6].copy_from_slice(&chunk_normal);
            result[6..8].copy_from_slice(&chunk_tex);
            result
        })
        .collect();

    //Convert vertex_positions from Vec<u32> to Vec<[u32;3]>
    let vertex_indices: Vec<VertIndicies> = models[0]
        .mesh
        .indices
        .chunks(3)
        .map(|chunk| TryInto::<[u32; 3]>::try_into(chunk).unwrap())
        .collect();

    Mesh {
        vertices,
        indicies: vertex_indices,
        tangents: Vec::new(),
        bi_tangents: Vec::new(),
        vao: None,
        vbo: None,
        ebo: None,
    }
}

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn update_model_matrices(&mut self) {
        for mesh in self.objects.iter_mut() {
            mesh.update_model_matrix();
        }
    }

    pub fn draw(&self) {
        for object in self.objects.iter() {
            object.mesh.draw();
        }
    }

    pub unsafe fn setup(&mut self) {
        for object in self.objects.iter_mut() {
            object.mesh.setup();
        }
    }
}

//High level object that contains a mesh and a transform
pub struct Object {
    pub mesh: Mesh,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,

    pub model_matrix: Matrix4<f32>,
}

impl Object {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh,
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            model_matrix: Matrix4::identity(),
        }
    }

    pub fn update_model_matrix(&mut self) {
        self.model_matrix = Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
    }
}

pub type VertIndicies = [u32; 3];

pub type Vertex = [f32; 3 + 3 + 2 + 3 + 3]; //Position, Normal, TextureCoords, Tangent, BiTangent

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indicies: Vec<VertIndicies>,
    pub tangents: Vec<Vector3<f32>>,
    pub bi_tangents: Vec<Vector3<f32>>,
    pub vao: Option<VertexArray>,
    pub vbo: Option<Buffer>,
    pub ebo: Option<Buffer>,
}

impl Mesh {
    pub fn new(v: Vec<Vertex>, i: Vec<VertIndicies>) -> Self {
        Self {
            vertices: v,
            indicies: i,
            tangents: Vec::new(),
            bi_tangents: Vec::new(),
            vao: None,
            vbo: None,
            ebo: None,
        }
    }

    pub unsafe fn calculate_tangents(&mut self) {

        //Calculate tangents and bi-tangents for each triangle
        for i in (0..self.indicies.len()) {
            let i0 = self.indicies[i][0] as usize;
            let i1 = self.indicies[i][1] as usize;
            let i2 = self.indicies[i][2] as usize;

            let v0 = Vector3::new(
                self.vertices[i0][0],
                self.vertices[i0][1],
                self.vertices[i0][2],
            );
            let v1 = Vector3::new(
                self.vertices[i1][0],
                self.vertices[i1][1],
                self.vertices[i1][2],
            );
            let v2 = Vector3::new(
                self.vertices[i2][0],
                self.vertices[i2][1],
                self.vertices[i2][2],
            );

            let normal = Vector3::new(
                self.vertices[i0][3],
                self.vertices[i0][4],
                self.vertices[i0][5],
            );


            let uv0 = Vector2::new(self.vertices[i0][6], self.vertices[i0][7]);
            let uv1 = Vector2::new(self.vertices[i1][6], self.vertices[i1][7]);
            let uv2 = Vector2::new(self.vertices[i2][6], self.vertices[i2][7]);

            let delta_pos1 = v1 - v0;
            let delta_pos2 = v2 - v0;

            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let mut tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;
            //Make tangent and bi-tangent orthogonal to the normal
            tangent = (tangent - normal * normal.dot(tangent)).normalize();

            //Put the tangent and bi-tangent in the vertices
            self.vertices[i0][8] = tangent.x;
            self.vertices[i0][9] = tangent.y;
            self.vertices[i0][10] = tangent.z;

            self.vertices[i1][8] = tangent.x;
            self.vertices[i1][9] = tangent.y;
            self.vertices[i1][10] = tangent.z;

            self.vertices[i2][8] = tangent.x;
            self.vertices[i2][9] = tangent.y;
            self.vertices[i2][10] = tangent.z;

            self.vertices[i0][11] = bitangent.x;
            self.vertices[i0][12] = bitangent.y;
            self.vertices[i0][13] = bitangent.z;
            
            self.vertices[i1][11] = bitangent.x;
            self.vertices[i1][12] = bitangent.y;
            self.vertices[i1][13] = bitangent.z;

            self.vertices[i2][11] = bitangent.x;
            self.vertices[i2][12] = bitangent.y;
            self.vertices[i2][13] = bitangent.z;
        }
    }

    pub unsafe fn setup(&mut self) -> &Self {
        self.calculate_tangents();

        self.vao = Some(VertexArray::new().expect("Failed to create vertex array"));

        self.vbo = Some(
            Buffer::new(GL_STATIC_DRAW, GL_ARRAY_BUFFER).expect("Failed to create vertex buffer"),
        );

        self.ebo = Some(
            Buffer::new(GL_STATIC_DRAW, GL_ELEMENT_ARRAY_BUFFER)
                .expect("Failed to create element buffer"),
        );

        //self.vao, and the other buffers, are an Option so we need to unwrap it
        self.vao.unwrap().bind();
        self.vbo.unwrap().bind();

        self.vbo
            .unwrap()
            .set_data(cast_slice(self.vertices.as_slice()));

        self.ebo.unwrap().bind();
        self.ebo
            .unwrap()
            .set_data(cast_slice(self.indicies.as_slice()));

        //Calculate tangents and bi-tangents

        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::ptr::null(),
        );
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(
            1,
            3,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 3]>() as *const _,
        );
        glEnableVertexAttribArray(1);

        glVertexAttribPointer(
            2,
            2,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 6]>() as *const _,
        );
        glEnableVertexAttribArray(2);

        glVertexAttribPointer(
            3,
            3,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 8]>() as *const _,
        );
        glEnableVertexAttribArray(3);

        glVertexAttribPointer(
            4,
            3,
            GL_FLOAT,
            GL_FALSE,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 11]>() as *const _,
        );
        glEnableVertexAttribArray(4);

        self.vao.unwrap().unbind();

        self
    }
    pub fn draw(&self) {
        unsafe {
            //Print the combined lengths of the vertices and indicies

            self.vao.unwrap().bind();
            glDrawElements(
                GL_TRIANGLES,
                self.indicies.len() as i32 * 3,
                GL_UNSIGNED_INT,
                std::ptr::null(),
            );
            self.vao.unwrap().unbind();
        }
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

    pub fn rotate(&mut self, angle: f32, axis: Vector3<f32>) {
        let rot = Quaternion::from_axis_angle(axis, Rad(angle));
        self.target = rot * (self.target - self.position) + self.position;
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
