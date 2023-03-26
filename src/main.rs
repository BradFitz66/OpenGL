#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![allow(special_module_name)]
const WINDOW_TITLE: &str = "'Hello world!' said the triangle";

use beryllium::*;
use bytemuck::bytes_of;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};
use imagine::{image::Bitmap, pixel_formats::RGBA8888};
use OpenGL::*;

use cgmath::{num_traits::clamp, *};
use cstr::*;
use ogl33::*;
use std::ffi::CStr;
use std::{any::Any, collections::HashMap, ffi::CString, path::Path};

//Ping pong a number between a min and max value
fn ping_pong(t: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    let phase = (t / range).floor();
    let t2 = t - phase * range;
    if (phase as i32) % 2 == 0 {
        min + t2
    } else {
        max - t2
    }
}

#[derive(Debug, Clone, Copy)]
struct PointLight {
    position: Vector4<f32>,
    color: Vector4<f32>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vector4::new(0.0, 0.0, 0.0, 0.0),
            color: Vector4::new(1.0, 1.0, 1.0, 0.0),
        }
    }
}

//Implement the bytemuck trait for the Light struct
unsafe impl bytemuck::Zeroable for PointLight {}
unsafe impl bytemuck::Pod for PointLight {}

fn main() {
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");

    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 4).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();

    let mut win = sdl
        .create_gl_window(
            WINDOW_TITLE,
            WindowPosition::Centered,
            800,
            600,
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");

    win.set_swap_interval(SwapInterval::Immediate);
    let mut shader_program;

    let camera = Camera::new(Vector3::new(0.0, 1.0, 3.0), Vector3::new(0.0, 0.0, 0.0));

    let mut plane_object = Object::new(mesh_from_obj(Path::new("assets/models/plane.obj")));
    let mut sphere_object = Object::new(mesh_from_obj(Path::new("assets/models/StandfordDragon.obj")));

    let bitmap_diffuse = {
        let mut f =
            std::fs::File::open(Path::new("assets/textures/Tiles094_1K_Color.png")).unwrap();
        let mut bytes = vec![];
        std::io::Read::read_to_end(&mut f, &mut bytes).unwrap();
        let bitmap = Bitmap::<RGBA8888>::try_from_png_bytes(&bytes).unwrap();

        bitmap
    };

    let bitmap_roughness = {
        let mut f =
            std::fs::File::open(Path::new("assets/textures/Tiles094_1K_Roughness.png")).unwrap();
        let mut bytes = vec![];
        std::io::Read::read_to_end(&mut f, &mut bytes).unwrap();
        let bitmap = Bitmap::<RGBA8888>::try_from_png_bytes(&bytes).unwrap();

        bitmap
    };
    let mut diffuse_texture = 0;
    let mut roughness_texture = 0;

    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.392, 0.584, 0.929, 1.0);
        glEnable(GL_DEPTH_TEST);

        sphere_object.mesh.setup();
        plane_object.mesh.setup();

    
        //ToDo: Wrapper for glGenTextures
        glGenTextures(1, &mut diffuse_texture);
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, diffuse_texture);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGBA8 as GLint,
            1024,
            1024,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            bitmap_diffuse.pixels.as_ptr().cast(),
        );
        glGenerateMipmap(GL_TEXTURE_2D);

        glGenTextures(1, &mut roughness_texture);
        glActiveTexture(GL_TEXTURE1);
        glBindTexture(GL_TEXTURE_2D, roughness_texture);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_R8 as GLint,
            1024,
            1024,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            bitmap_roughness.pixels.as_ptr().cast(),
        );
        glGenerateMipmap(GL_TEXTURE_2D);
        

        shader_program = ShaderProgramBuilder::new()
            .create_shader(
                ShaderType::Vertex,
                &shader_from_file(Path::new("assets/shaders/vertex_shader.vert")),
            )
            .create_shader(
                ShaderType::Fragment,
                &shader_from_file(Path::new("assets/shaders/fragment_shader.frag")),
            )
            .link()
            .unwrap();
        shader_program.create_uniform(cstr!("M"));
        shader_program.create_uniform(cstr!("V"));
        shader_program.create_uniform(cstr!("P"));
        shader_program.create_uniform(cstr!("albedo"));
        shader_program.create_uniform(cstr!("roughness"));
        shader_program.create_uniform(cstr!("metallic"));
        shader_program.create_uniform(cstr!("camera_pos"));
        shader_program.create_uniform(cstr!("diffuse_map"));
        shader_program.create_uniform(cstr!("roughness_map"));

        glUseProgram(shader_program.0);

    }

    'main_loop: loop {
        let frame_start = sdl.get_ticks();

        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                _ => (),
            }
        }
        let time = sdl.get_ticks() as f32 / 10.0_f32;
        let translation_pingpong = ping_pong(time * 0.01, -1.0, 1.0);
        let roughness_pingpong = ping_pong(time * 0.01, 0.0, 1.0);
        let transform;

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0));
            shader_program.set_mat4("M", &transform);
            shader_program.set_mat4("V", &camera.get_view_matrix());
            shader_program.set_mat4("P", &camera.get_projection_matrix());
            shader_program.set_vec3("camera_pos", &camera.position);
            shader_program.set_vec3("albedo", &Vector3::new(1.0,1.0,1.0));
            shader_program.set_float("roughness", roughness_pingpong);

            shader_program.set_int("diffuse_map", 0);
            shader_program.set_int("roughness_map", 1);

            sphere_object.mesh.draw();

            shader_program.set_vec3("albedo", &Vector3::new(1.0,1.0,1.0));
            plane_object.mesh.draw();
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
    }
}
