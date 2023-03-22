#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![allow(special_module_name)]
const WINDOW_TITLE: &str = "'Hello world!' said the triangle";


use OpenGL::*;
use beryllium::*;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};

use ogl33::*;
use std::{any::Any, ffi::CString, path::Path};
use cgmath::*;
use std::ffi::CStr;
use cstr::*;

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;


  uniform mat4 transform;
  out vec2 tex_coord;

  void main() {
    //gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    gl_Position = transform * vec4(pos,1.0);
    tex_coord = vec2(pos.x, pos.y);
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  uniform vec4 uni_color;
  out vec4 final_color;
  in vec2 tex_coord;
  void main() {
    //Set final color to gl_FragCoord
    final_color = uni_color;
  }
"#;

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
    let mut tri_mesh:Mesh;
    let mut shader_program;
    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.392, 0.584, 0.929, 1.0);
        tri_mesh=Mesh::new(
            vec![
                Vertex([0.0,0.5,0.0],[0.0,0.0,0.0],[0.0,0.5]),
                Vertex([0.5,-0.5,0.0],[0.0,0.0,0.0],[0.5,-0.5]),
                Vertex([-0.5,-0.5,0.0],[0.0,0.0,0.0],[-0.5,-0.5]),
            ],
            vec![[0, 1, 2]],
        );
        tri_mesh.setup();   

        shader_program = ShaderProgramBuilder::new()
            .create_shader(ShaderType::Vertex,VERT_SHADER)
            .create_shader(ShaderType::Fragment,FRAG_SHADER)
            .link()
            .unwrap();

        shader_program.create_uniform(cstr!("transform"));
        shader_program.create_uniform(cstr!("uni_color"));
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
        let transform = Matrix4::from_angle_z(Deg(time));

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT);
            let transform_name: *const c_char  = cstr!("transform").as_ptr().cast();
            
            shader_program.set_mat4("transform", &transform);
            shader_program.set_vec4("uni_color", &Vector4::new(1.0, 0.0, 0.0, 1.0));
            
            glDrawElements(GL_TRIANGLES, 3, GL_UNSIGNED_INT, std::ptr::null());
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
    }
}
