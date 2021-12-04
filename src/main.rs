use std::f32::consts::TAU;

use cgmath::{Angle, Deg, Matrix4, vec3};
use glium::{Blend, Display, DrawParameters, Frame, IndexBuffer, Program, ProgramCreationError, Surface, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{KeyboardInput, ModifiersState, MouseScrollDelta, StartCause};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use msgbox::IconType;

use crate::window::{Context, create, Handler};

mod window;

#[derive(Copy, Clone, glium_derive::Vertex)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
    uv: [f32; 2]
}

struct WindowContext {
    program: Program,
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u16>,
    scale: f32,
    angle_y: f32,
    angle_x: f32,
    width: f32,
    height: f32,
}

fn compile_program(display: &Display, vertex: &str, fragment: &str, geometry: Option<&str>) -> Program {
    match Program::from_source(display, vertex, fragment, geometry) {
        Ok(program) => program,
        Err(e) => {
            match e {
                ProgramCreationError::CompilationError(message, shader) => {
                    let message = format!("Error compiling {:?} Shader from source:\n\n{}", shader, message);
                    msgbox::create("Shader compilation error", &message, IconType::Error);
                    Err(ProgramCreationError::CompilationError(message, shader)).unwrap()
                }
                other => {
                    Err(other).unwrap()
                }
            }
        }
    }
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let vertex = include_str!("shaders/flat.vert");
        let fragment = include_str!("shaders/flat.frag");
        let program = compile_program(display, vertex, fragment, None);

        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let (vertices, indices) = cube(display);
        let indices = IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();
        Self {
            program,
            vertices,
            indices,
            scale: 20.0,
            angle_y: 0.0,
            angle_x: 0.0,
            width: size.width,
            height: size.height,
        }
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, frame: &mut Frame, time_elapsed: f32) {
        let view = cgmath::ortho(0.0, context.width, context.height, 0.0, -1024.0, 1024.0);
        let center = vec3(context.width / 2.0, context.height / 2.0, 0.0);
        let model = Matrix4::from_translation(center)
            * Matrix4::from_scale(context.scale)
            * Matrix4::from_angle_y(Deg(context.angle_y))
            * Matrix4::from_angle_x(Deg(context.angle_x));

        let matrix: [[f32;4];4] = (view * model).into();

        let data = uniform! {
            matrix: matrix,
            time: time_elapsed
        };
        let params = DrawParameters {
            line_width: Some(15.0),
            blend: Blend::alpha_blending(),
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            ..Default::default()
        };

        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        frame.draw(&context.vertices, &context.indices, &context.program, &data, &params).unwrap();
    }

    fn on_resized(&mut self, context: &mut WindowContext, width: f32, height: f32) {
        context.width = width;
        context.height = height;
    }

    fn on_mouse_scroll(&mut self, context: &mut WindowContext, delta: MouseScrollDelta, modifiers: ModifiersState) {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                if modifiers.ctrl() {
                    context.scale += 10.0 * y;
                } else if modifiers.shift() {
                    context.angle_x += 10.0 * y;
                } else {
                    context.angle_y += 10.0 * y;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(800, 600), 24, WindowHandler);
}

fn triangle(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        Vertex { pos: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        Vertex { pos: [ 1.0,  1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] },
    ]).unwrap();
    let indices = vec![0, 1, 2];
    (vertices, indices)
}

fn square(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        Vertex { pos: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        Vertex { pos: [ 1.0,  1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        Vertex { pos: [ 1.0, -1.0, 0.0], color: [0.0, 0.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] }
    ]).unwrap();
    let indices = vec![0, 1, 2, 3];
    (vertices, indices)
}

fn circle(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(360);
    let mut indices = Vec::with_capacity(360);
    for i in 0..360 {
        let angle = Deg(i as f32);
        let x = angle.cos();
        let y = angle.sin();
        vertices.push(Vertex { pos: [x, y, 0.0], color: [1.0, 1.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [(x + 1.0) / 2.0, (y + 1.0) / 2.0] });
        indices.push(i as u16);
    }
    let vertices = VertexBuffer::new(display, &vertices).unwrap();
    (vertices, indices)
}

fn cube(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let vertices = VertexBuffer::new(display, &[
        // Max X
        Vertex { pos: [ 0.5, -0.5, -0.5], normal: [1.0,  0.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5,  0.5], normal: [1.0,  0.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5,  0.5], normal: [1.0,  0.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, -0.5], normal: [1.0,  0.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Min X
        Vertex { pos: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5,  0.5], normal: [-1.0, 0.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Max Y
        Vertex { pos: [-0.5,  0.5, -0.5], normal: [0.0,  1.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, -0.5], normal: [0.0,  1.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5,  0.5], normal: [0.0,  1.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5,  0.5], normal: [0.0,  1.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Min Y
        Vertex { pos: [-0.5, -0.5, -0.5], normal: [0.0, -1.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5, -0.5,  0.5], normal: [0.0, -1.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Max Z
        Vertex { pos: [-0.5, -0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Min Z
        Vertex { pos: [-0.5, -0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
    ]).unwrap();
    let mut indices = Vec::new();
    for face in 0..6u16 {
        for i in &[0, 1, 2, 0, 2, 3] {
            indices.push(4 * face + *i);
        }
    }
    (vertices, indices)
}