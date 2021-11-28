use std::time::{Duration, Instant};
use cgmath::{Deg, Matrix4};
use glium::{Display, DrawParameters, IndexBuffer, Program, Surface, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{ElementState, Event, KeyboardInput, MouseScrollDelta, StartCause, VirtualKeyCode, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;

#[derive(Copy, Clone, glium_derive::Vertex)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4]
}

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Разработка игр")
        .with_inner_size(LogicalSize::new(800, 800));

    let cb = ContextBuilder::new().with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let vertex = include_str!("shaders/triangle.vert");
    let fragment = include_str!("shaders/triangle.frag");
    let program = Program::from_source(&display, vertex, fragment, None).unwrap();

    let (vertices, indices) = triangle(&display);

    let start_time = Instant::now();

    let mut scale = 0.5;
    let mut angle = 0.0;

    event_loop.run(move |event, _, control_flow| {
        let next_frame = Instant::now() + Duration::from_secs_f32(1.0 / 60.0);
        *control_flow = ControlFlow::WaitUntil(next_frame);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::KeyboardInput { input, .. } if input.state == ElementState::Pressed => {
                    if let Some(code) = input.virtual_keycode {
                        match code {
                            VirtualKeyCode::Return => {
                                println!("Нажали Enter");
                            }
                            _ => {}
                        }
                    }
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            // angle += y;
                            scale += y / 100.0;
                        },
                        _ => {}
                    }
                }
                _ => {
                    *control_flow = ControlFlow::Poll;
                },
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => {},
                StartCause::Init => {},
                _ => return,
            }
            _ => return,
        };

        let now = Instant::now();
        let time_elapsed = now.duration_since(start_time).as_secs_f32();
        // angle = time_elapsed * 20.0;

        let matrix = Matrix4::from_angle_z(Deg(angle))
            * Matrix4::from_scale(scale)
            * cgmath::ortho(0.0, 400.0, 400.0, 0.0, -1.0, 1.0);

        let data = uniform! {
            matrix: Into::<[[f32;4];4]>::into(matrix),
            time: time_elapsed
        };
        let params = DrawParameters::default();

        let mut target = display.draw();

        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        target.draw(&vertices, &indices, &program, &data, &params).unwrap();

        target.finish().unwrap();
    });
}

fn triangle(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [0.0,   0.0,    0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [0.0,   400.0,  0.0], color: [0.0, 1.0, 0.0, 1.0] },
        Vertex { pos: [400.0, 400.0,  0.0], color: [0.0, 0.0, 1.0, 1.0] },
    ]).unwrap();
    let indices = IndexBuffer::new(display, PrimitiveType::LineLoop, &[
        0, 1, 2
    ]).unwrap();
    (vertices, indices)
}

fn square(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [0.0,   0.0,    0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [0.0,   400.0,  0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [400.0, 400.0,  0.0], color: [1.0, 0.0, 0.0, 1.0] },

        Vertex { pos: [400.0, 400.0,  0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [400.0, 0.0,    0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [0.0,   0.0,    0.0], color: [1.0, 0.0, 0.0, 1.0] },
    ]).unwrap();
    let indices = IndexBuffer::new(display, PrimitiveType::TrianglesList, &[
        0, 1, 2,
        3, 4, 5
    ]).unwrap();
    (vertices, indices)
}