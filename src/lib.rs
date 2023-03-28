use __core::ffi::c_char;
use __core::ffi::c_int;
use __core::ffi::c_uchar;
use __core::ffi::c_uint;
use bytemuck::*;
use glam::{Vec3, Quat, Mat4, Vec2};
use gl33::*;
use gl33::global_loader::*;
use stb_image::image::{LoadResult, Image};

use std::os::raw::c_void;
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
};
use std::{fs::File, io::prelude::*, path::Path};
//Wrapper for opengl textures
#[derive(Clone)]
pub struct Texture2D<T>{
    pub id: c_uint,
    pub data: Vec<T>,
}

impl Texture2D<u8> {
    pub unsafe fn new(texture_unit: GLenum,image_path:&str, gl:&GlFns) -> Option<Self> {
        let path = Path::new(image_path);
        let mut data:Vec<u8> = Vec::new();
        let result:LoadResult = stb_image::image::load_with_depth(path, 0, true);

        match result {
            LoadResult::ImageU8(image) => {
                data = image.data;
            }
            LoadResult::ImageF32(_image) => {
                panic!("Tried to load F32 image as U8");
            }
            LoadResult::Error(error) => {
                println!("Error loading image: {}", error);
                return None;
            }
        }

        let mut texture = 0;
        gl.GenTextures(1, &mut texture);
        gl.ActiveTexture(texture_unit);
        if texture == 0 {
            None
        } else {
            Some(Self{id:texture,data:data})
        }
    }

    pub unsafe fn bind(&self,gl:&GlFns) {
        gl.BindTexture(GL_TEXTURE_2D, self.id);
    }

    pub unsafe fn unbind(&self,gl:&GlFns) {
        gl.BindTexture(GL_TEXTURE_2D, 0);
    }

    pub unsafe fn set_data(
        &self,
        width: i32,
        height: i32,
        format: GLenum,
        internal_format: c_int,
        type_: GLenum,
        gl:&GlFns
    ) {
        gl.TexImage2D(
            GL_TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format,
            type_,
            self.data.as_ptr() as *const c_void,
        );
    }

    pub unsafe fn set_filter(&self, filter: GLenum,gl:&GlFns) {
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filter.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filter.0 as i32);
    }

    pub unsafe fn set_wrap(&self, wrap: GLenum,gl:&GlFns) {
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap.0 as i32);
    }
}

impl Texture2D<f32> {
    pub unsafe fn new(texture_unit: GLenum,image_path:&str, gl:&GlFns) -> Option<Self> {
        let path = Path::new(image_path);
        let mut data:Vec<f32> = Vec::new();
        let result:LoadResult = stb_image::image::load(path);

        match result {
            LoadResult::ImageU8(_image) => {
                panic!("Tried to load U8 image as F32");
            }
            LoadResult::ImageF32(image) => {
                data = image.data;
            }
            LoadResult::Error(error) => {
                panic!("Error loading image: {}", error);
            }
        }

        let mut texture = 0;
        gl.GenTextures(1, &mut texture);
        gl.ActiveTexture(texture_unit);
        if texture == 0 {
            None
        } else {
            Some(Self{id:texture,data:data})
        }
    }

    pub unsafe fn bind(&self,gl:&GlFns) {
        gl.BindTexture(GL_TEXTURE_2D, self.id);
    }

    pub unsafe fn unbind(&self,gl:&GlFns) {
        gl.BindTexture(GL_TEXTURE_2D, 0);
    }

    pub unsafe fn set_data(
        &self,
        width: i32,
        height: i32,
        internal_format: c_int,
        format: GLenum,
        type_: GLenum,
        gl:&GlFns
    ) {
        gl.TexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGB16F.0 as i32,
            width,
            height,
            0,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            self.data.as_ptr() as *const c_void,
        );
    }

    pub unsafe fn set_filter(&self, filter: GLenum,gl:&GlFns) {
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filter.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filter.0 as i32);
    }

    pub unsafe fn set_wrap(&self, wrap: GLenum,gl:&GlFns) {
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap.0 as i32);
    }
}

//Wrapper for opengl buffer objects
#[derive(Clone, Copy)]
pub struct Buffer(pub c_uint, pub GLenum, pub GLenum); //Buffer, Target, Usage

impl Buffer {
    pub unsafe fn new(usage: GLenum, target: GLenum,gl:&GlFns) -> Option<Self> {
        let mut buffer = 0;
        gl.GenBuffers(1, &mut buffer);

        if buffer == 0 {
            None
        } else {
            Some(Self(buffer, target, usage))
        }
    }

    pub unsafe fn bind(&self,gl:&GlFns) -> &Self {
        gl.BindBuffer(self.1, self.0);
        self
    }

    pub unsafe fn unbind(&self,gl:&GlFns) -> &Self {
        gl.BindBuffer(self.1, 0);
        self
    }

    pub unsafe fn set_data(&self, data: &[u8],gl:&GlFns) {
        gl.BufferData(
            self.1,
            data.len().try_into().unwrap(),
            data.as_ptr() as *const _,
            self.2,
        );
    }
}

//Buffer for opengl vertex array objects
#[derive(Clone, Copy)]
pub struct VertexArray(pub c_uint);

impl VertexArray {
    pub unsafe fn new(gl:&GlFns) -> Option<Self> {
        let mut vao = 0;

        gl.GenVertexArrays(1, &mut vao);

        if vao == 0 {
            None
        } else {
            Some(Self(vao))
        }
    }

    pub unsafe fn unbind(&self,gl:&GlFns) {
        gl.BindVertexArray(0);
    }

    pub unsafe fn bind(&self,gl:&GlFns) {
        gl.BindVertexArray(self.0);
    }
}

pub enum ShaderType {
    Vertex = GL_VERTEX_SHADER.0 as isize ,
    Fragment = GL_FRAGMENT_SHADER.0 as isize,
}

pub struct ShaderProgramBuilder {
    id: c_uint,
    uniforms: HashMap<String, c_int>,
    uniformblocks: HashMap<String, c_uint>,
}

//Wrapper for opengl shader programs (uses builder pattern)
pub struct ShaderProgram(
    pub c_uint,
    pub HashMap<String, c_int>,
    pub HashMap<String, c_uint>,
); //Program, Uniforms, UniformBlocks

impl ShaderProgram {
    pub unsafe fn create_uniform(&mut self, name: &CStr, gl:&GlFns) {
        let uniform_location = gl.GetUniformLocation(self.0, name.as_ptr() as *const u8);
        self.1
            .insert(name.to_str().unwrap().to_string(), uniform_location);
    }

    pub unsafe fn create_uniformblock(&mut self, name: &CStr, gl:&GlFns) {
        let uniform_location = gl.GetUniformBlockIndex(self.0, name.as_ptr() as *const u8);
        self.2
            .insert(name.to_str().unwrap().to_string(), uniform_location);
    }

    pub unsafe fn set_mat4(&self, name: &str, mat: &Mat4, gl:&GlFns) {
        gl.UniformMatrix4fv(
            self.1[name],
            1,
            GL_FALSE.0 as u8,
            mat as *const Mat4 as *const f32,
        );
    }

    pub unsafe fn set_vec3(&self, name: &str, vec: Vec3, gl:&GlFns) {
        gl.Uniform3f(self.1[name], vec.x, vec.y, vec.z);
    }

    pub unsafe fn set_vec4(&self, name: &str, vec: &cgmath::Vector4<f32>, gl:&GlFns) {
        gl.Uniform4f(self.1[name], vec.x, vec.y, vec.z, vec.w);
    }

    pub unsafe fn set_float(&self, name: &str, val: f32, gl:&GlFns) {
        gl.Uniform1f(self.1[name], val);
    }

    pub unsafe fn set_int(&self, name: &str, val: i32, gl:&GlFns) {
        gl.Uniform1i(self.1[name], val);
    }
}

impl ShaderProgramBuilder {
    pub fn new(gl:&GlFns) -> Self {
        let id = gl.CreateProgram();
        Self {
            id: id,
            uniforms: HashMap::new(),
            uniformblocks: HashMap::new(),
        }
        
    }
    //Compiles shader from source and attaches it to the program, as-well as deleting it after attachment
    pub fn create_shader(&self, shader_type: ShaderType, shader_src: &str, gl:&GlFns) -> &Self {
        unsafe {            
            let shader = compile_shader(shader_type, shader_src,gl).expect("Failed to compile shader");
            //Print GL errors
            gl.AttachShader(self.id, shader);
            gl.DeleteShader(shader);
        }
        self
    }
    //Links the program and returns a ShaderProgram wrapper
    pub fn link(&self, gl:&GlFns) -> Option<ShaderProgram> {
        unsafe {
            gl.LinkProgram(self.id);

            let mut success = 0;
            gl.GetProgramiv(self.id, GL_LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;

                gl.GetProgramInfoLog(self.id, 1024, &mut log_len, v.as_mut_ptr().cast());
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
unsafe fn compile_shader(shader_type: ShaderType, shader_src: &str, gl:&GlFns) -> Option<u32> {
    println!("Entering compile_shader");
    let shader = gl.CreateShader(gl33::GLenum(shader_type as u32));
    let shader_src = CString::new("").unwrap();
    
    gl.ShaderSource(shader, 1, shader_src.as_ptr() as *const *const c_uchar, std::ptr::null());
    
    gl.CompileShader(shader);
    let mut success = 0;
    gl.GetShaderiv(shader, GL_COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut v: Vec<u8> = Vec::with_capacity(1024);
        let mut log_len = 0_i32;

        gl.GetShaderInfoLog(shader, 1024, &mut log_len, v.as_mut_ptr().cast());
        v.set_len(log_len.try_into().unwrap());
        panic!(
            "Shader compilation failed: {}",
            String::from_utf8(v).unwrap()
        );
    }

    Some(shader)
}


//Parse an obj file and return a mesh (HEAVILY WIP)
pub fn mesh_from_obj(path: &Path, ) -> Mesh {
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

    pub fn draw(&self, gl: &GlFns) {
        for object in self.objects.iter() {
            object.mesh.draw(gl);
        }
    }

    pub unsafe fn setup(&mut self, gl: &GlFns) {
        for object in self.objects.iter_mut() {
            object.mesh.setup(gl);
        }
    }
}

//High level object that contains a mesh and a transform
pub struct Object {
    pub mesh: Mesh,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    pub model_matrix: Mat4,
}

impl Object {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh,
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            model_matrix: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }
    }

    pub fn update_model_matrix(&mut self) {
        self.model_matrix = Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale);
    }
}

pub type VertIndicies = [u32; 3];

pub type Vertex = [f32; 3 + 3 + 2 + 3 + 3]; //Position, Normal, TextureCoords, Tangent, BiTangent

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indicies: Vec<VertIndicies>,
    pub tangents: Vec<Vec3>,
    pub bi_tangents: Vec<Vec3>,
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

            let v0 = Vec3::new(
                self.vertices[i0][0],
                self.vertices[i0][1],
                self.vertices[i0][2],
            );
            let v1 = Vec3::new(
                self.vertices[i1][0],
                self.vertices[i1][1],
                self.vertices[i1][2],
            );
            let v2 = Vec3::new(
                self.vertices[i2][0],
                self.vertices[i2][1],
                self.vertices[i2][2],
            );

            let normal = Vec3::new(
                self.vertices[i0][3],
                self.vertices[i0][4],
                self.vertices[i0][5],
            );

            let uv0 = Vec2::new(self.vertices[i0][6], self.vertices[i0][7]);
            let uv1 = Vec2::new(self.vertices[i1][6], self.vertices[i1][7]);
            let uv2 = Vec2::new(self.vertices[i2][6], self.vertices[i2][7]);

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

    pub unsafe fn setup(&mut self, gl:&GlFns) -> &Self {
        self.calculate_tangents();

        self.vao = Some(VertexArray::new(gl).expect("Failed to create vertex array"));

        self.vbo = Some(
            Buffer::new(GL_STATIC_DRAW, GL_ARRAY_BUFFER,gl).expect("Failed to create vertex buffer"),
        );

        self.ebo = Some(
            Buffer::new(GL_STATIC_DRAW, GL_ELEMENT_ARRAY_BUFFER,gl)
                .expect("Failed to create element buffer"),
        );

        //self.vao, and the other buffers, are an Option so we need to unwrap it
        self.vao.unwrap().bind(gl);
        self.vbo.unwrap().bind(gl);

        self.vbo
            .unwrap()
            .set_data(cast_slice(self.vertices.as_slice()),gl);

        self.ebo.unwrap().bind(gl);
        self.ebo
            .unwrap()
            .set_data(cast_slice(self.indicies.as_slice()),gl);

        //Calculate tangents and bi-tangents

        gl.VertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::ptr::null(),
        );
        gl.EnableVertexAttribArray(0);

        gl.VertexAttribPointer(
            1,
            3,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 3]>() as *const _,
        );
        gl.EnableVertexAttribArray(1);

        gl.VertexAttribPointer(
            2,
            2,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 6]>() as *const _,
        );
        gl.EnableVertexAttribArray(2);

        gl.VertexAttribPointer(
            3,
            3,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 8]>() as *const _,
        );
        gl.EnableVertexAttribArray(3);

        gl.VertexAttribPointer(
            4,
            3,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            std::mem::size_of::<Vertex>().try_into().unwrap(),
            std::mem::size_of::<[f32; 11]>() as *const _,
        );
        gl.EnableVertexAttribArray(4);

        self.vao.unwrap().unbind(gl);

        self
    }
    pub fn draw(&self, gl:&GlFns) {
        unsafe {
            //Print the combined lengths of the vertices and indicies

            self.vao.unwrap().bind(gl);
            gl.DrawElements(
                GL_TRIANGLES,
                self.indicies.len() as i32 * 3,
                GL_UNSIGNED_INT,
                std::ptr::null(),
            );
            self.vao.unwrap().unbind(gl);
        }
    }
}

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub fov: f32,
    pub aspect: [f32; 2],
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(start_pos: Vec3, targ: Vec3) -> Self {
        Self {
            position: start_pos,
            target: targ,
            fov: 45.0,
            aspect: [4.0, 3.0],
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn get_direction(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    pub fn get_right(&self) -> Vec3 {
        self.get_direction()
            .cross(Vec3::new(0.0, 1.0, 0.0))
            .normalize()
    }

    pub fn get_up(&self) -> Vec3 {
        self.get_right().cross(self.get_direction()).normalize()
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn rotate(&mut self, angle: f32, axis: Vec3) {
        let rot = Quat::from_axis_angle(axis, angle);
        self.target = rot * (self.target - self.position) + self.position;
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let pos_p = Vec3::new(self.position.x, self.position.y, self.position.z);
        let target_p = Vec3::new(self.target.x, self.target.y, self.target.z);
        let up_v = self.get_up();
        let up_p = Vec3::new(up_v.x, up_v.y, up_v.z);
        Mat4::look_at_rh(pos_p, target_p, up_p)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(
            self.fov, 
            self.aspect[0] / self.aspect[1],
            self.near, 
            self.far
        )
    }
}
