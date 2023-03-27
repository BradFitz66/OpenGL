#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![allow(special_module_name)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_assignments)]

const WINDOW_TITLE: &str = "'Hello world!' said the triangle";

use beryllium::*;

use bytemuck::bytes_of;

use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};

use imagine::{image::Bitmap, pixel_formats::RGBA8888};
use OpenGL_Renderer::*;

use ogl33::*;
use cgmath::{num_traits::clamp, *};

use colored::*;
use cstr::*;

use std::{any::Any, 
    collections::HashMap, 
    ffi::CString, 
    ffi::CStr,
    path::Path,
    fmt::{self, Display},
    time::Instant,
};

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

#[derive(Debug, Clone, Copy)]
enum TraceLevel {
    ERROR,
    WARNING,
    INFO,
}

fn trace(message: &str, level: TraceLevel) {
    let prefix = "TRACE_";
    let level_str = match level {
        TraceLevel::ERROR => "ERROR",
        TraceLevel::WARNING => "WARNING",
        TraceLevel::INFO => "INFO",
    };
    match level {
        TraceLevel::ERROR => println!("{}{}: {}", prefix.red(), level_str.red(), message.white()),
        TraceLevel::WARNING => println!(
            "{}{}: {}",
            prefix.yellow(),
            level_str.yellow(),
            message.white()
        ),
        TraceLevel::INFO => println!(
            "{}{}: {}",
            prefix.green(),
            level_str.green(),
            message.white()
        ),
    }
}

fn load_image(path: &Path) -> Bitmap<RGBA8888> {
    let mut f = std::fs::File::open(path).unwrap();
    let mut bytes = vec![];
    std::io::Read::read_to_end(&mut f, &mut bytes).unwrap();
    let bitmap = Bitmap::<RGBA8888>::try_from_png_bytes(&bytes).unwrap();

    bitmap
}

fn main() {
    //For timing execution (i.e. how long it takes to load assets)
    let timer;
    let mut mouse_captured: bool = true;
    let mut movement: [bool; 4] = [false; 4]; //[forward, backward, left, right]
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

    trace("Window created", TraceLevel::INFO);
    win.set_swap_interval(SwapInterval::Vsync);

    let mut shader_program;
    let mut camera = Camera::new(Vector3::new(0.0, 1.0, 3.0), Vector3::new(0.0, 0.0, 0.0));

    timer = Instant::now();
    
    let mut plane_object = Object::new(mesh_from_obj(Path::new("assets/models/plane.obj")));
    let mut sphere_object = Object::new(mesh_from_obj(Path::new("assets/models/sphere.obj")));
    

    let bitmap_diffuse = load_image(Path::new("assets/textures/DiamondPlate008C_1K_Color.png"));
    let bitmap_roughness = load_image(Path::new("assets/textures/DiamondPlate008C_1K_Roughness.png"));
    let bitmap_metallic = load_image(Path::new("assets/textures/DiamondPlate008C_1K_Metalness.png"));
    let bitmap_normal = load_image(Path::new("assets/textures/DiamondPlate008C_1K_NormalGL.png"));

    


    trace(
        &format!("Assets loaded in {:?}", timer.elapsed()),
        TraceLevel::INFO,
    );


    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
        glClearColor(0.392, 0.584, 0.929, 1.0);
        glEnable(GL_DEPTH_TEST);

        sphere_object.mesh.setup();
        plane_object.mesh.setup();

        let diffuse_map = Texture2D::new(GL_TEXTURE0).expect("Couldn't create texture");
        diffuse_map.bind();
        diffuse_map.set_wrap(GL_REPEAT);
        diffuse_map.set_filter(GL_LINEAR);
        diffuse_map.set_data(
            bytemuck::cast_slice(&bitmap_diffuse.pixels),
            1024,
            1024,
            GL_RGBA,
            GL_RGBA as GLint,
            GL_UNSIGNED_BYTE,
        );

        let roughness_map = Texture2D::new(GL_TEXTURE1).expect("Couldn't create texture");
        roughness_map.bind();
        roughness_map.set_wrap(GL_REPEAT);
        roughness_map.set_filter(GL_LINEAR);
        roughness_map.set_data(
            bytemuck::cast_slice(&bitmap_roughness.pixels),
            1024,
            1024,
            GL_RGBA,
            GL_RGBA as GLint,
            GL_UNSIGNED_BYTE,
        );

        let normal_map = Texture2D::new(GL_TEXTURE2).expect("Couldn't create texture");
        normal_map.bind();
        normal_map.set_wrap(GL_REPEAT);
        normal_map.set_filter(GL_LINEAR);
        normal_map.set_data(
            bytemuck::cast_slice(&bitmap_normal.pixels),
            1024,
            1024,
            GL_RGBA,
            GL_RGBA as GLint,
            GL_UNSIGNED_BYTE,
        );

        let metallic_map = Texture2D::new(GL_TEXTURE3).expect("Couldn't create texture");
        metallic_map.bind();
        metallic_map.set_wrap(GL_REPEAT);
        metallic_map.set_filter(GL_LINEAR);
        metallic_map.set_data(
            bytemuck::cast_slice(&bitmap_metallic.pixels),
            1024,
            1024,
            GL_RGBA,
            GL_RGBA as GLint,
            GL_UNSIGNED_BYTE,
        );


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
        shader_program.create_uniform(cstr!("normal_map"));
        shader_program.create_uniform(cstr!("metallic_map"));

        glUseProgram(shader_program.0);
    }
    sdl.set_relative_mouse_mode(mouse_captured)
        .expect("Couldn't set relative mouse mode");
    let mut frame_start = sdl.get_ticks();
    let mut last_frame = 0;
    'main_loop: loop {
        frame_start = sdl.get_ticks();
        let delta_time: f32 = ((frame_start as f32) - (last_frame as f32)) / 1000.0;
        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                Event::Keyboard(KeyboardEvent {
                    key, is_pressed, ..
                }) => {
                    if key.keycode == Keycode::ESCAPE && is_pressed {
                        //Toggle cursor lock
                        mouse_captured = !mouse_captured;
                        sdl.set_relative_mouse_mode(mouse_captured)
                            .expect("Couldn't set relative mouse mode");
                    }
                    if key.keycode == Keycode::W && is_pressed {
                        movement[0] = true;
                    } else if key.keycode == Keycode::W && !is_pressed {
                        movement[0] = false;
                    }

                    if key.keycode == Keycode::S && is_pressed {
                        movement[1] = true;
                    } else if key.keycode == Keycode::S && !is_pressed {
                        movement[1] = false;
                    }

                    if key.keycode == Keycode::A && is_pressed {
                        movement[2] = true;
                    } else if key.keycode == Keycode::A && !is_pressed {
                        movement[2] = false;
                    }

                    if key.keycode == Keycode::D && is_pressed {
                        movement[3] = true;
                    } else if key.keycode == Keycode::D && !is_pressed {
                        movement[3] = false;
                    }
                }
                //Mouse movement
                Event::MouseMotion(MouseMotionEvent {
                    x_delta, y_delta, ..
                }) => {
                    if !mouse_captured  {
                        continue;
                    }
                    camera.rotate(
                        -((x_delta as f32) * delta_time * 0.1) as f32,
                        Vector3::new(0.0, 1.0, 0.0),
                    );
                    camera.rotate(-((y_delta as f32) * delta_time * 0.1) as f32, camera.get_right());
                }
                Event::MouseButton(MouseButtonEvent {
                    button, is_pressed, ..
                }) => {
                    if button == MouseButton::Left && is_pressed {
                        //Set cursor lock to true
                        mouse_captured = true;
                        sdl.set_relative_mouse_mode(mouse_captured)
                            .expect("Couldn't set relative mouse mode");
                    }
                }
                _ => (),
            }
        }
        let time = sdl.get_ticks() as f32 / 10.0_f32;
        let roughness_pingpong = ping_pong(time * 0.01, 0.0, 1.0);
        let transform;
        if movement[0] {
            camera.set_position(camera.position + camera.get_direction() * delta_time * 5.0);
            camera.target += camera.get_direction() * delta_time * 5.0;
        }
        if movement[1] {
            camera.set_position(camera.position - camera.get_direction() * delta_time * 5.0);
            camera.target -= camera.get_direction() * delta_time * 5.0;
        }
        if movement[2] {
            camera.set_position(camera.position - camera.get_right() * delta_time * 5.0);
            camera.target -= camera.get_right() * delta_time * 5.0;
        }
        if movement[3] {
            camera.set_position(camera.position + camera.get_right() * delta_time * 5.0);
            camera.target += camera.get_right() * delta_time * 5.0;
        }

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0));
            shader_program.set_mat4("M", &transform);
            shader_program.set_mat4("V", &camera.get_view_matrix());
            shader_program.set_mat4("P", &camera.get_projection_matrix());
            shader_program.set_vec3("camera_pos", &camera.position);
            shader_program.set_vec3("albedo", &Vector3::new(1.0, 1.0, 1.0));
            shader_program.set_float("roughness", 0.25);
            shader_program.set_float("metallic", 1.0);

            shader_program.set_int("diffuse_map", 0);
            shader_program.set_int("roughness_map", 1);
            shader_program.set_int("normal_map", 2);
            shader_program.set_int("metallic_map", 3);

            sphere_object.mesh.draw();

            shader_program.set_vec3("albedo", &Vector3::new(1.0, 1.0, 1.0));
            plane_object.mesh.draw();
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
        last_frame = frame_start;
    }
}
