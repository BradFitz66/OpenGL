#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![allow(special_module_name)]
const WINDOW_TITLE: &str = "'Hello world!' said the triangle";

use beryllium::*;
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

// // Loops the value t, so that it is never larger than length and never smaller than 0.
// public static float Repeat(float t, float length)
// {
//     return Clamp(t - Mathf.Floor(t / length) * length, 0.0f, length);
// }


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
    let mut rect_mesh: Mesh;
    let mut shader_program;
    let mut camera = Camera::new(Vector3::new(0.0, 2.0, 5.0), Vector3::new(0.0, 0.0, 0.0));
    let mut test_scene = Scene::new();
    let plane_object = Object::new(mesh_from_obj(Path::new("assets/models/plane.obj")));
    let monke_object = Object::new(mesh_from_obj(Path::new("assets/models/sphere.obj")));

    test_scene.add_object(plane_object);
    test_scene.add_object(monke_object);


    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.392, 0.584, 0.929, 1.0);
        glEnable(GL_DEPTH_TEST);

        test_scene.setup();

        shader_program = ShaderProgramBuilder::new()
            .create_shader(ShaderType::Vertex, &shader_from_file(Path::new("assets/shaders/vertex_shader.vert")))
            .create_shader(ShaderType::Fragment, &shader_from_file(Path::new("assets/shaders/fragment_shader.frag")))
            .link()
            .unwrap();

        shader_program.create_uniform(cstr!("MVP"));
        shader_program.create_uniform(cstr!("uni_color"));
        shader_program.create_uniform(cstr!("V"));
        shader_program.create_uniform(cstr!("M"));
        shader_program.create_uniform(cstr!("M_V"));
        shader_program.create_uniform(cstr!("light_pos"));
        shader_program.create_uniform(cstr!("light_color"));


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
        let transform = Matrix4::from_value(1.0);
        let rot_speed = 0.01;
        //Rotate camera around the triangle with a speed of rot_speed
        // camera.set_position(Vector3::new(
        //     4.0 * (time * rot_speed).sin(),
        //     2.0,
        //     4.0 * (time * rot_speed).cos(),
        // ));
        //Ping pong a number between 0 and 1
        let ping_pong = ping_pong(time*0.01, -1.0, 1.0);
        let light_pos = Vector3::new(ping_pong, 1.0, 0.0);
        
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            let mvp = camera.get_projection_matrix() * camera.get_view_matrix() * transform;
            shader_program.set_mat4("MVP", &mvp);
            shader_program.set_mat4("V", &camera.get_view_matrix());
            shader_program.set_mat4("M", &transform);
            shader_program.set_vec3("M_V", &camera.position);
            shader_program.set_vec3("light_pos", &light_pos);
            shader_program.set_vec3("uni_color", &Vector3::new(1.0, 1.0, 0.0));  
            shader_program.set_vec3("light_color", &Vector3::new(0.0,1.0,0.0));
            test_scene.draw();
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
    }
}
