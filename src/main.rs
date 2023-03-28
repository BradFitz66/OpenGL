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
use ogl33::c_uint;

use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};

use glam::*;

use OpenGL_Renderer::*;

use gl33::global_loader::*;
use gl33::*;

use colored::*;
use cstr::*;
use std::{
    any::Any,
    collections::HashMap,
    ffi::CStr,
    ffi::{c_int, CString,c_void, c_uchar},
    fmt::{self, Display},
    path::Path,
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
    position: Vec4,
    color: Vec4,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec4::new(0.0, 0.0, 0.0, 0.0),
            color: Vec4::new(1.0, 1.0, 1.0, 0.0),
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

// fn load_hdr_image(path: &Path) -> Vec<f32> {
//     let mut f = std::fs::File::open(path).unwrap();
//     let f = std::io::BufReader::new(f);

//     let pixels:Vec<[f32;3]> = radiant::load(f).unwrap().data.into_iter().map(|rgb|{
//         [rgb.r, rgb.g, rgb.b]
//     }).collect();
//}

/*

the type constructed contains 
    fn(Option<unsafe extern "system" fn(
    GLenum, 
    GLenum, 
    u32, 
    GLenum, 
    i32, 
    *const u8, 
    *const c_void)>
) 
{debug_callback}
    expected fn pointer `unsafe extern "system" 
    fn(gl33::GLenum, gl33::GLenum, _, gl33::GLenum, _, _, _)`
    fn(gl33::GLenum, gl33::GLenum, _, gl33::GLenum, _, _, _)`

 */

unsafe extern "system" fn debug_callback(
    source: GLenum,
    type_: GLenum,
    id: c_uint,
    severity: GLenum,
    length: c_int,
    message: *const c_uchar,
    userparam: *const c_void,

) {
    let  src:&str;

    match source {
        GL_DEBUG_SOURCE_API => src = "API",
        GL_DEBUG_SOURCE_WINDOW_SYSTEM => src = "WINDOW_SYSTEM",
        GL_DEBUG_SOURCE_SHADER_COMPILER => src = "SHADER_COMPILER",
        GL_DEBUG_SOURCE_THIRD_PARTY => src = "THIRD_PARTY",
        GL_DEBUG_SOURCE_APPLICATION => src = "APPLICATION",
        GL_DEBUG_SOURCE_OTHER => src = "OTHER",
        _ => src = "UNKNOWN",
    }

    let t:&str;

    match type_ {
        gl33::GL_DEBUG_TYPE_ERROR => t = "ERROR",
        gl33::GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR => t = "DEPRECATED_BEHAVIOR",
        gl33::GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR => t = "UNDEFINED_BEHAVIOR",
        gl33::GL_DEBUG_TYPE_PORTABILITY => t = "PORTABILITY",
        gl33::GL_DEBUG_TYPE_PERFORMANCE => t = "PERFORMANCE",
        gl33::GL_DEBUG_TYPE_MARKER => t = "MARKER",
        gl33::GL_DEBUG_TYPE_PUSH_GROUP => t = "PUSH_GROUP",
        gl33::GL_DEBUG_TYPE_POP_GROUP => t = "POP_GROUP",
        gl33::GL_DEBUG_TYPE_OTHER => t = "OTHER",
        _ => t = "UNKNOWN",
    }


    let sev:&str;

    match severity {
        gl33::GL_DEBUG_SEVERITY_HIGH => sev = "HIGH",
        gl33::GL_DEBUG_SEVERITY_MEDIUM => sev = "MEDIUM",
        gl33::GL_DEBUG_SEVERITY_LOW => sev = "LOW",
        gl33::GL_DEBUG_SEVERITY_NOTIFICATION => sev = "NOTIFICATION",
        _ => sev = "UNKNOWN",
    }
    let level = match severity {
        gl33::GL_DEBUG_SEVERITY_HIGH => TraceLevel::ERROR,
        gl33::GL_DEBUG_SEVERITY_MEDIUM => TraceLevel::WARNING,
        gl33::GL_DEBUG_SEVERITY_LOW => TraceLevel::INFO,
        gl33::GL_DEBUG_SEVERITY_NOTIFICATION => TraceLevel::INFO,
        _ => TraceLevel::INFO,
    };

  


    trace(
        &format!(
            "GL CALLBACK: {:?} type = {}, severity = {}, message = {:?}",
            src,
            t,
            sev,
            CStr::from_ptr(message as *const i8)
        ),
        level,
    );
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
    let mut camera = Camera::new(Vec3::new(0.0, 1.0, 3.0), Vec3::new(0.0, 0.0, 0.0));

    timer = Instant::now();

    let mut plane_object = Object::new(mesh_from_obj(Path::new("assets/models/plane.obj")));
    let mut sphere_object = Object::new(mesh_from_obj(Path::new("assets/models/monke.obj")));

    //let skybox = load_hdr_image(Path::new("assets/textures/kloofendal_48d_partly_cloudy_puresky_2k.hdr"));
    let gl: GlFns;
    unsafe {
        gl = GlFns::load_from(&|c_char_ptr| {
            win.get_proc_address(c_char_ptr as *const i8) as *const _
        })
        .unwrap();

        gl.ClearColor(0.392, 0.584, 0.929, 1.0);
        gl.Enable(GL_DEPTH_TEST);
        gl.Enable(GL_DEBUG_OUTPUT);
        gl.DebugMessageCallback(
            Some(debug_callback),
            std::ptr::null::<c_void>() as *const c_void,
        );
        trace("OpenGL context created", TraceLevel::INFO);
        sphere_object.mesh.setup(&gl);
        plane_object.mesh.setup(&gl);

        let diffuse_map = Texture2D::<u8>::new(
            GL_TEXTURE0,
            "assets/textures/DiamondPlate008C_1K_Color.png",
            &gl,
        )
        .expect("Couldn't create texture");
        diffuse_map.bind(&gl);
        diffuse_map.set_wrap(GL_REPEAT, &gl);
        diffuse_map.set_filter(GL_LINEAR, &gl);
        diffuse_map.set_data(1024, 1024, GL_RGB, GL_RGB.0 as c_int, GL_UNSIGNED_BYTE, &gl);

        let roughness_map = Texture2D::<u8>::new(
            GL_TEXTURE1,
            "./assets/textures/DiamondPlate008C_1K_Roughness.png",
            &gl,
        )
        .expect("Couldn't create texture");
        roughness_map.bind(&gl);
        roughness_map.set_wrap(GL_REPEAT, &gl);
        roughness_map.set_filter(GL_LINEAR, &gl);
        roughness_map.set_data(1024, 1024, GL_RED, GL_RED.0 as c_int, GL_UNSIGNED_BYTE, &gl);

        let normal_map = Texture2D::<u8>::new(
            GL_TEXTURE2,
            "assets/textures/DiamondPlate008C_1K_NormalGL.png",
            &gl,
        )
        .expect("Couldn't create texture");
        normal_map.bind(&gl);
        normal_map.set_wrap(GL_REPEAT, &gl);
        normal_map.set_filter(GL_LINEAR, &gl);
        normal_map.set_data(1024, 1024, GL_RGB, GL_RGB.0 as c_int, GL_UNSIGNED_BYTE, &gl);

        let metallic_map = Texture2D::<u8>::new(
            GL_TEXTURE3,
            "assets/textures/DiamondPlate008C_1K_Metalness.png",
            &gl,
        )
        .expect("Couldn't create texture");
        metallic_map.bind(&gl);
        metallic_map.set_wrap(GL_REPEAT, &gl);
        metallic_map.set_filter(GL_LINEAR, &gl);
        metallic_map.set_data(1024, 1024, GL_RED, GL_RED.0 as c_int, GL_UNSIGNED_BYTE, &gl);

        let skybox_map = Texture2D::<f32>::new(GL_TEXTURE4, "assets/textures/hotel.hdr", &gl)
            .expect("Couldn't create texture");
        skybox_map.bind(&gl);
        skybox_map.set_wrap(GL_CLAMP_TO_EDGE, &gl);
        skybox_map.set_filter(GL_LINEAR, &gl);
        skybox_map.set_data(2048, 2048, GL_RGB.0 as c_int, GL_RGB, GL_UNSIGNED_BYTE, &gl);

        trace(
            &format!("Assets loaded in {:?}", timer.elapsed()),
            TraceLevel::INFO,
        );

        shader_program = ShaderProgramBuilder::new(&gl)
            .create_shader(
                OpenGL_Renderer::ShaderType::Vertex,
                &shader_from_file(Path::new("assets/shaders/vertex_shader.vert")),
                &gl,
            )
            .create_shader(
                OpenGL_Renderer::ShaderType::Fragment,
                &shader_from_file(Path::new("assets/shaders/fragment_shader.frag")),
                &gl,
            )
            .link(&gl)
            .unwrap();
        trace(&format!("Shader program created"), TraceLevel::INFO);
        shader_program.create_uniform(cstr!("M"), &gl);
        shader_program.create_uniform(cstr!("V"), &gl);
        shader_program.create_uniform(cstr!("P"), &gl);
        shader_program.create_uniform(cstr!("albedo"), &gl);
        shader_program.create_uniform(cstr!("roughness"), &gl);
        shader_program.create_uniform(cstr!("metallic"), &gl);
        shader_program.create_uniform(cstr!("camera_pos"), &gl);
        shader_program.create_uniform(cstr!("diffuse_map"), &gl);
        shader_program.create_uniform(cstr!("roughness_map"), &gl);
        shader_program.create_uniform(cstr!("normal_map"), &gl);
        shader_program.create_uniform(cstr!("metallic_map"), &gl);
        shader_program.create_uniform(cstr!("equirectangular_map"), &gl);
        gl.UseProgram(shader_program.0);
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
                    if !mouse_captured {
                        continue;
                    }
                    camera.rotate(
                        -((x_delta as f32) * delta_time * 0.1) as f32,
                        Vec3::new(0.0, 1.0, 0.0),
                    );
                    camera.rotate(
                        -((y_delta as f32) * delta_time * 0.1) as f32,
                        camera.get_right(),
                    );
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
            camera.set_position(camera.position + camera.get_direction() * delta_time * 3.0);
            camera.target += camera.get_direction() * delta_time * 3.0;
        }
        if movement[1] {
            camera.set_position(camera.position - camera.get_direction() * delta_time * 3.0);
            camera.target -= camera.get_direction() * delta_time * 3.0;
        }
        if movement[2] {
            camera.set_position(camera.position - camera.get_right() * delta_time * 3.0);
            camera.target -= camera.get_right() * delta_time * 3.0;
        }
        if movement[3] {
            camera.set_position(camera.position + camera.get_right() * delta_time * 3.0);
            camera.target += camera.get_right() * delta_time * 3.0;
        }

        unsafe {
            gl.Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            transform = Mat4::from_rotation_y(time * 0.01);
            shader_program.set_mat4("M", &transform, &gl);
            shader_program.set_mat4("V", &camera.get_view_matrix(), &gl);
            shader_program.set_mat4("P", &camera.get_projection_matrix(), &gl);
            shader_program.set_vec3("camera_pos", camera.position, &gl);
            shader_program.set_vec3("albedo", Vec3::new(1.0, 0.0, 0.0), &gl);
            shader_program.set_float("metallic", 1.0, &gl);

            shader_program.set_int("diffuse_map", 0, &gl);
            shader_program.set_int("roughness_map", 1, &gl);
            shader_program.set_int("normal_map", 2, &gl);
            shader_program.set_int("metallic_map", 3, &gl);
            shader_program.set_int("equirectangular_map", 4, &gl);

            shader_program.set_float("roughness", 1.0, &gl);

            sphere_object.mesh.draw(&gl);

            shader_program.set_vec3("albedo", Vec3::new(1.0, 1.0, 1.0), &gl);
            //plane_object.mesh.draw();
        }
        win.swap_window();
        let msec = sdl.get_ticks() - frame_start;
        if msec > 0 {
            win.set_title(&format!("{} - FPS: {}", WINDOW_TITLE, 1000 / msec));
        }
        last_frame = frame_start;
    }
}
