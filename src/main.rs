use std::{time, env, fs};

use glium::{
    glutin::{
        dpi::PhysicalSize,
        event::{self, ElementState, MouseButton}, window::Fullscreen,
    },
    texture::{MipmapsOption, UncompressedFloatFormat}, Surface, BlitTarget,
};
use yaml_rust::YamlLoader;

use crate::{board::{random_board_binary, empty_board, random_board}, program::{val_program::ValProgram, Program, rgb_program::RgbProgram}};

mod board;
mod program;
fn main() {
    use glium::glutin;

    let args: Vec<String> = env::args().collect();
    
    let doc = if let Some(path) = args.get(1) {
        let string = fs::read_to_string(path).unwrap();
        let docs = YamlLoader::load_from_str(&string).unwrap();
        docs[0].to_owned()
    } else {
        println!("Provide a program .yaml file.");
        return;
    };

    let height = doc["screen"][1].as_i64().unwrap().try_into().unwrap();
    let width = doc["screen"][0].as_i64().unwrap().try_into().unwrap();
    let aspect_ratio = width as f32 / height as f32;

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        // .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(None)))
        .with_min_inner_size(PhysicalSize::new(width, height))
        .with_inner_size(PhysicalSize::new(width, height))
        
        ;
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let program = RgbProgram::from_yaml(&doc, &display);

    

    let mut board = glium::texture::Texture2d::with_format(
        &display,
        vec![vec![(0., 0., 0., 1.); width as usize]; height as usize],
        UncompressedFloatFormat::F32F32F32F32,
        MipmapsOption::NoMipmap,
    )
    .unwrap();

    let mut mouse_pressed = (false, false);
    let mut active_color = ActiveColor::Red;

    let mut last_frame_instant = time::Instant::now();
    let mut last_frame_time = time::Duration::ZERO;

    let mut speed = 32;
    let mut step_counter = 0;
    
    let mut draw_queue = Vec::<(u32, u32)>::new();

    event_loop.run(move |ev, _, control_flow| {
        if last_frame_instant.elapsed() >= time::Duration::from_nanos(16_666_667) {
            last_frame_instant = time::Instant::now();

            if speed < 32 {
                step_counter += 1;
                if step_counter % (32 / speed) == 0 {
                    program.step(&mut board);
                }
            } else {
                for _ in 1..=speed / 32 {
                    program.step(&mut board);
                }
            }

            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);

            {
                let dim = target.get_dimensions();
                let real_ratio = dim.0 as f32 / dim.1 as f32;

                let (height, width) = if real_ratio > aspect_ratio {
                    let h = dim.1;
                    (h, (h as f32 * aspect_ratio) as u32)
                } else {
                    let w = dim.0;
                    ((w as f32 / aspect_ratio) as u32, w)
                };
                board
                .as_surface()
                .blit_whole_color_to(
                    &target, 
                    &BlitTarget {
                        left: (dim.0 - width) / 2,
                        bottom: (dim.1 - height) / 2,
                        width: width as i32,
                        height: height as i32
                    }, 
                    glium::uniforms::MagnifySamplerFilter::Nearest);
            }
            target.finish().unwrap();

            if !draw_queue.is_empty() {
                
                let mut buffer: Vec<Vec<(u8, u8, u8, u8)>> = board.read();
                
                // let mut map_write = buffer.map_write();
                for (x, y) in &draw_queue {
                    buffer[*y as usize][*x as usize] = match active_color {
                        ActiveColor::Red => (255, 0, 0, 255),
                        ActiveColor::Green => (0, 255, 0, 255),
                        ActiveColor::Blue => (0, 0, 255, 255),
                        ActiveColor::White => (255, 255, 255, 255),
                    }
                }
                board = glium::texture::Texture2d::with_format(
                    &display,
                    buffer,
                    UncompressedFloatFormat::F32F32F32F32,
                    MipmapsOption::NoMipmap,
                )
                .unwrap();
                draw_queue.truncate(0);
            }

            last_frame_time = last_frame_instant.elapsed();
            if last_frame_time > time::Duration::from_nanos(16_666_667) {
                // println!("{}ms frame time!", last_frame_time.as_millis())
            }

            
        }

        // println!("{} ms/f", next_frame_time.duration_since(last_frame_time).as_millis());
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                event::WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) => mouse_pressed.0 = true,
                    (ElementState::Released, MouseButton::Left) => mouse_pressed.0 = false,
                    (ElementState::Pressed, MouseButton::Right) => mouse_pressed.1 = true,
                    (ElementState::Released, MouseButton::Right) => mouse_pressed.1 = false,
                    _ => (),
                },
                event::WindowEvent::CursorMoved { position, .. } => {
                    if mouse_pressed.0 {
                        let inner_size = display.gl_window().window().inner_size();
                        let x: u32 = (position.x as u32 * width) / inner_size.width;
                        let t: u32 = (position.y as u32 * height) / inner_size.height;
                        let y: u32 = height
                            - t.min(height);
                        // println!("mp: {} {}, inner_size: {} {}, board_size: {} {}, xy: {} {}", mouse_pos.x, mouse_pos.y, inner_size.width, inner_size.height, width, height, x, y);
                        draw_queue.push((x.min(width - 1), y.min(height - 1)));
                    }
                }
                event::WindowEvent::KeyboardInput {
                    input,
                    ..
                } => {
                    if input.state == ElementState::Pressed {
                        match input.scancode {
                            57 => {
                                // space
                                board = glium::texture::Texture2d::with_format(
                                    &display,
                                    random_board_binary(width, height),
                                    UncompressedFloatFormat::F32F32F32F32,
                                    MipmapsOption::NoMipmap,
                                )
                                .unwrap();
                            }
                            45 => {
                                // x
                                board = glium::texture::Texture2d::with_format(
                                    &display,
                                    random_board(width, height),
                                    UncompressedFloatFormat::F32F32F32F32,
                                    MipmapsOption::NoMipmap,
                                )
                                .unwrap();
                            }
                            46 => {
                                // c
                                board = glium::texture::Texture2d::with_format(
                                    &display,
                                    empty_board(width, height),
                                    UncompressedFloatFormat::F32F32F32F32,
                                    MipmapsOption::NoMipmap,
                                )
                                .unwrap();
                            }
                            19 | 34 | 48 | 2 => {
                                // r, g, b, w
                                match input.scancode {
                                    34 => active_color = ActiveColor::Green,
                                    48 => active_color = ActiveColor::Blue,
                                    2 => active_color = ActiveColor::White,
                                    _ => active_color = ActiveColor::Red
                                }
                            }
                            33 => {
                                // f
                                println!("frametime: {}ms", last_frame_time.as_millis());
                            }
                            1 | 16 => {
                                // esc | q
                                *control_flow = glutin::event_loop::ControlFlow::Exit;
                            }
                            87 => {
                                // F11
                                let gl_window = display.gl_window();
                                let window = gl_window.window();
                                if window.fullscreen().is_none() {
                                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                } else {
                                    window.set_fullscreen(None);
                                }
                                
                            }
                            x if x >= 2 && x <= 11 => { 
                                // 1..0
                                let mode = x - 2;
                                speed = 1 << mode;
                            }
                            x => println!("{}", x),
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });
}

enum ActiveColor {
    Red, Green, Blue, White
}