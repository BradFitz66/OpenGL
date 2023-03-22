#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![allow(special_module_name)]
const WINDOW_TITLE: &str = "'Hello world!' said the triangle";

mod lib;

use beryllium::*;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};

use crate::lib::{
    buffer::*,
    mesh::*,
    shader::{ShaderProgram, ShaderProgramBuilder, ShaderType},
    vertex_array::VertexArray,
    objparser::*
};

use self::lib::*;

use ogl33::*;
use std::{any::Any, ffi::CString, path::Path};
use cgmath::*;
use std::ffi::CStr;
use cstr::*;

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;


  uniform mat4 model;
  uniform mat4 view;
  uniform mat4 projection;

  out vec2 tex_coord;

  void main() {
    //gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    gl_Position = projection * view * model * vec4(pos,1.0);
    tex_coord = vec2(pos.x, pos.y);
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 final_color;
  in vec2 tex_coord;
  void main() {
    //Set final color to gl_FragCoord
    final_color = vec4(gl_FragCoord.x / 800.0, gl_FragCoord.y / 600.0, 0.0, 1.0);
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
    let mut rect_mesh: Mesh;

    //let test = ParseOBJ(Path::new("./assets/Cube.obj"));
    let view: Matrix4<f32> = Matrix4::from_translation(vec3(0., 0., 0.));
    let projection: Matrix4<f32> = perspective(Deg(45.0), 800 as f32 / 600 as f32, 0.1, 100.0);    
    let model: Matrix4<f32>= Matrix4::from_translation(vec3(0., 0., 0.));

    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.2, 0.3, 0.3, 1.0);
        glEnable(GL_DEPTH_TEST);
        //Create vertex array object
        rect_mesh = Mesh::new(
            //Rectangle vertices
            vec![
                Vertex([-0.5, -0.5, 0.0], [0.0, 0.0, 0.0], [-0.5, -0.5]),
                Vertex([0.5, -0.5, 0.0], [0.0, 0.0, 0.0], [0.5, -0.5]),
                Vertex([0.5, 0.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5]),
                Vertex([-0.5, 0.5, 0.0], [0.0, 0.0, 0.0], [-0.5, 0.5]),
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3]
            ],
        );
        rect_mesh.setup();

        //Build shader program (is this ugly?)
        let shader_program = ShaderProgramBuilder::new()
            .attach_shader(ShaderType::Vertex, VERT_SHADER)
            .attach_shader(ShaderType::Fragment, FRAG_SHADER)
            .link()
            .expect("Failed to build shader program");

        glUseProgram(shader_program.0);
        let modelLoc = glGetUniformLocation(shader_program.0, cstr!("model").as_ptr());
        let viewLoc = glGetUniformLocation(shader_program.0, cstr!("view").as_ptr());
        glUniformMatrix4fv(modelLoc, 1, GL_FALSE, &(model[0][0] as f32));
        glUniformMatrix4fv(viewLoc, 1, GL_FALSE, &(view[0][0] as f32));
        shader_program.set_mat4(cstr!("projection"), &projection);
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
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT);
            glDrawElements(
                GL_TRIANGLES,
                (rect_mesh.vertices.len() + rect_mesh.indices.len())
                    .try_into()
                    .unwrap(),
                GL_UNSIGNED_INT,
                0 as *const _,
            );
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
    }
}
