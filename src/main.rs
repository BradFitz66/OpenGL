#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]

const WINDOW_TITLE: &str = "Triangle: Draw Arrays Cleaned Up";
mod lib;


use beryllium::*;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};

use crate::lib::{
    shader::{ShaderProgram, ShaderType, ShaderProgramBuilder},
    buffer::*,
};

use self::lib::*;

use ogl33::*;
use std::{ffi::CString, any::Any};

type Vertex = [f32; 3];

const TRIANGLE: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

const CUBE: [Vertex; 8] = [ 
    // front
    [-0.5, -0.5,  0.5],
    [ 0.5, -0.5,  0.5],
    [ 0.5,  0.5,  0.5],
    [-0.5,  0.5,  0.5],
    // back
    [-0.5, -0.5, -0.5],
    [ 0.5, -0.5, -0.5],
    [ 0.5,  0.5, -0.5],
    [-0.5,  0.5, -0.5]
];

const CUBE_INDICES: [GLushort; 36] = [
    // front
    0, 1, 2,
    2, 3, 0,
    // right
    1, 5, 6,
    6, 2, 1,
    // back
    7, 6, 5,
    5, 4, 7,
    // left
    4, 0, 3,
    3, 7, 4,
    // bottom
    4, 5, 1,
    1, 0, 4,
    // top
    3, 2, 6,
    6, 7, 3,
];

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;

  void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 final_color;

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

    let win = sdl
        .create_gl_window(
            WINDOW_TITLE,
            WindowPosition::Centered,
            800,
            600,
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    win.set_swap_interval(SwapInterval::Vsync);

    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.2, 0.3, 0.3, 1.0);

        //Create vertex array object
        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        glBindVertexArray(vao);
        assert_ne!(vao, 0);

        //Create vertex buffer object
        let buffer = Buffer::new().expect("Failed to create buffer");
        buffer.bind(GL_ARRAY_BUFFER);

        //Copy vertex data to buffer
        glBufferData(
            GL_ARRAY_BUFFER,
            size_of_val(&TRIANGLE).try_into().unwrap(),
            TRIANGLE.as_ptr() as *const _,
            GL_STATIC_DRAW,
        );

        //Set vertex attribute pointers
        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            std::ptr::null(),
        );
        glEnableVertexAttribArray(0);

        //Create a shader program (is this ugly?)
        let shader_program = ShaderProgramBuilder::new()
        .attach_shader(ShaderType::Vertex, VERT_SHADER)
        .attach_shader(ShaderType::Fragment, FRAG_SHADER)
        .link()
        .expect("Failed to link shader program");

        glUseProgram(shader_program.0);

    }

    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                _ => (),
            }
        }
        // now the events are clear.

        // here's where we could change the world state if we had some.

        // and then draw!
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLES, 0, 3);
        }
        win.swap_window();
    }
}
