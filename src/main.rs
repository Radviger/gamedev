use std::f32::consts::TAU;

use cgmath::{Angle, Deg, Matrix4, vec3};
use glium::{Blend, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{KeyboardInput, ModifiersState, MouseScrollDelta, StartCause};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;

use crate::MouseScrollDelta::LineDelta;
use crate::window::{Context, Handler};

mod window;

#[derive(Copy, Clone, glium_derive::Vertex)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}

struct WindowContext {
    program: Program,
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u16>,
    scale: f32,
    angle: f32,
    width: f32,
    height: f32,
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let vertex = include_str!("shaders/flat.vert");
        let fragment = include_str!("shaders/flat.frag");
        let program = Program::from_source(display, vertex, fragment, None).unwrap();

        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let (vertices, indices) = triangle(&display);
        Self {
            program,
            vertices,
            indices,
            scale: 200.0,
            angle: 0.0,
            width: size.width,
            height: size.height,
        }
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, frame: &mut Frame, time_elapsed: f32) {
        let view = cgmath::ortho(0.0, context.width, context.height, 0.0, -1.0, 1.0);
        let model = Matrix4::from_translation(vec3(context.width / 2.0, context.height / 2.0, 0.0))
            * Matrix4::from_scale(context.scale)
            * Matrix4::from_angle_z(Deg(context.angle));

        let matrix: [[f32;4];4] = (view * model).into();

        let data = uniform! {
            matrix: matrix,
            time: time_elapsed
        };
        let params = DrawParameters {
            line_width: Some(15.0),
            blend: Blend::alpha_blending(),
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
            LineDelta(x, y) => {
                if modifiers.contains(ModifiersState::SHIFT) {
                    context.angle += 10.0 * y;
                } else {
                    context.scale += 10.0 * y;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(800, 800), 24, WindowHandler);
}

fn triangle(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
        Vertex { pos: [ 1.0,  1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0] },
    ]).unwrap();
    let indices = IndexBuffer::new(display, PrimitiveType::TrianglesList, &[
        0, 1, 2
    ]).unwrap();
    (vertices, indices)
}

fn square(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
        Vertex { pos: [ 1.0,  1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0] },
        Vertex { pos: [ 1.0, -1.0, 0.0], color: [0.0, 0.0, 0.0, 1.0] }
    ]).unwrap();
    let indices = IndexBuffer::new(display, PrimitiveType::TriangleFan, &[
        0, 1, 2, 3
    ]).unwrap();
    (vertices, indices)
}

fn circle(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let mut vertices = Vec::with_capacity(360);
    let mut indices = Vec::with_capacity(360);
    for i in 0..360 {
        let angle = Deg(i as f32);
        let x = angle.cos();
        let y = angle.sin();
        vertices.push(Vertex { pos: [x, y, 0.0], color: [1.0, 1.0, 1.0, 1.0] });
        indices.push(i as u16);
    }
    let vertices = VertexBuffer::new(display, &vertices).unwrap();
    let indices = IndexBuffer::new(display, PrimitiveType::TriangleFan, &indices).unwrap();
    (vertices, indices)
}