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
use OpenGL::*;

use cgmath::{*, num_traits::clamp};
use cstr::*;
use ogl33::*;
use std::ffi::CStr;
use std::{any::Any, ffi::CString, path::Path};


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
            position: Vector4::new(0.0, 0.0, 0.0,0.0),
            color: Vector4::new(1.0, 1.0, 1.0,0.0),
        }
    }
}

//Implement the bytemuck trait for the Light struct
unsafe impl bytemuck::Zeroable for PointLight {}
unsafe impl bytemuck::Pod for PointLight {}

fn main() {
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();
    #[cfg(target_os = "macos")]
    {
        sdl.gl_set_attribute(SdlGlAttr::Flags, ContextFlag::ForwardCompatible)
            .unwrap();
    }

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
    let camera = Camera::new(Vector3::new(0.0, 2.0, 5.0), Vector3::new(0.0, 0.0, 0.0));
    let mut plane_object = Object::new(mesh_from_obj(Path::new("assets/models/plane.obj")));
    let mut sphere_object = Object::new(mesh_from_obj(Path::new("assets/models/sphere.obj")));

    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.392, 0.584, 0.929, 1.0);
        glEnable(GL_DEPTH_TEST);

        sphere_object.mesh.setup();
        plane_object.mesh.setup();

        shader_program = ShaderProgramBuilder::new()
            .create_shader(ShaderType::Vertex, &shader_from_file(Path::new("assets/shaders/vertex_shader.vert")))
            .create_shader(ShaderType::Fragment, &shader_from_file(Path::new("assets/shaders/fragment_shader.frag")))
            .link()
            .unwrap();

        shader_program.create_uniform(cstr!("MVP"));
        shader_program.create_uniform(cstr!("V"));
        shader_program.create_uniform(cstr!("M"));
        shader_program.create_uniform(cstr!("M_V"));
        shader_program.create_uniform(cstr!("V_P"));
        shader_program.create_uniform(cstr!("albedo"));
        shader_program.create_uniform(cstr!("metallic"));
        shader_program.create_uniform(cstr!("roughness"));
        shader_program.create_uniform(cstr!("ao"));
        shader_program.create_uniform(cstr!("lights[0].position"));
        shader_program.create_uniform(cstr!("lights[0].color"));
        shader_program.create_uniform(cstr!("lights[1].position"));
        shader_program.create_uniform(cstr!("lights[1].color"));
        shader_program.create_uniform(cstr!("directional_light.direction"));
        shader_program.create_uniform(cstr!("directional_light.color"));
        
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
        let mut transform = Matrix4::from_value(1.0);

        let ping_pong = ping_pong(time*0.01, -1.0, 1.0);
        
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            let mut mvp = camera.get_projection_matrix() * camera.get_view_matrix() * transform;
            shader_program.set_mat4("MVP", &mvp);
            shader_program.set_mat4("V", &camera.get_view_matrix());
            shader_program.set_mat4("M", &transform);
            shader_program.set_vec3("M_V", &camera.position);
            shader_program.set_vec3("albedo", &Vector3::new(1.0, 0.0, 0.0));
            shader_program.set_float("metallic", 0.0);
            shader_program.set_float("roughness", 1.0);
            shader_program.set_vec4("directional_light.direction", &Vector4::new(ping_pong, 1.0, 0.0, 0.0));
            shader_program.set_vec4("directional_light.color", &Vector4::new(1.0, 1.0, 1.0, 1.0));

            shader_program.set_vec4("lights[0].position", &Vector4::new(ping_pong, 0.0, 1.0, 1.0));
            shader_program.set_vec4("lights[0].color", &Vector4::new(0.0, 0.0, 1.0,1.0));
            
            shader_program.set_vec4("lights[1].position", &Vector4::new(ping_pong, 0.0, -1.0, 1.0));
            shader_program.set_vec4("lights[1].color", &Vector4::new(1.0, 0.0, 0.0,1.0));

            sphere_object.mesh.draw();

            transform = Matrix4::from_translation(Vector3::new(ping_pong, 0.0, 0.0));
            mvp = camera.get_projection_matrix() * camera.get_view_matrix() * transform;

            shader_program.set_mat4("MVP", &mvp);
            shader_program.set_vec3("albedo", &Vector3::new(0.0, 1.0, 0.0));
            plane_object.mesh.draw();
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
    }
}
