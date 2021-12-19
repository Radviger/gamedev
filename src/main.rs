use std::sync::Arc;

use cgmath::{Deg, Matrix4, vec3};
use glium::{Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{ElementState, KeyboardInput, ModifiersState, MouseScrollDelta, StartCause, VirtualKeyCode};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::uniforms::MagnifySamplerFilter;

use crate::shapes::Vertex;
use crate::window::{Context, Handler};

mod window;
mod shaders;
mod shapes;
mod textures;

struct WindowContext {
    program: Program,
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u16>,
    scale: f32,
    angle_y: f32,
    angle_x: f32,
    width: f32,
    height: f32,
    color: [f32; 3],
    texture: Arc<Texture2d>,
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let vertex = include_str!("../resources/shaders/flat.vert");
        let fragment = include_str!("../resources/shaders/flat.frag");
        let program = shaders::compile(display, vertex, fragment, None);

        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let texture = textures::load(display, "resources/bricks.jpg");

        let (vertices, indices) = shapes::cube(display);
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
            color: [1.0, 0.0, 0.0],
            texture,
        }
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, frame: &mut Frame, time_elapsed: f32) {
        let view = cgmath::perspective(Deg(70.0), 1.0, 0.1, 1024.0) *
            cgmath::ortho(0.0, context.width, context.height, 0.0, -1024.0, 1024.0);
        let center = vec3(context.width / 2.0, context.height / 2.0, 512.0);
        let model = Matrix4::from_translation(center)
            * Matrix4::from_scale(context.scale)
            * Matrix4::from_angle_y(Deg(context.angle_y))
            * Matrix4::from_angle_x(Deg(context.angle_x));

        let matrix: [[f32; 4]; 4] = (view * model).into();

        let data = uniform! {
            matrix: matrix,
            time: time_elapsed,
            tex: context.texture.sampled().magnify_filter(MagnifySamplerFilter::Nearest)
        };
        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            multisampling: true,
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
            MouseScrollDelta::LineDelta(_, y) => {
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

    fn on_keyboard_input(&mut self, context: &mut WindowContext, input: KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            if key == VirtualKeyCode::Return && input.state == ElementState::Pressed {
                println!("Pressed enter!");
            }
        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(800, 600), 24, WindowHandler);
}