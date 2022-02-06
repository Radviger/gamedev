use std::detect::__is_feature_detected::sha;
use std::sync::Arc;
use std::time::Instant;

use cgmath::{Deg, Matrix4, SquareMatrix, vec3};
use glium::{Blend, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, StartCause, VirtualKeyCode};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::texture::SrgbTexture2d;
use glium::uniforms::MagnifySamplerFilter;
use crate::audio::SoundSystem;
use crate::font::{FontParameters, TextAlignHorizontal};
use crate::render::Canvas;

use crate::window::{Context, Handler};

#[macro_use]
extern crate glium;

mod window;
mod shaders;
mod textures;
mod render;
mod font;
mod audio;

struct WindowContext {
    start: Instant,
    display: Arc<Display>,
    width: f32,
    height: f32,
    color: [f32; 3],
    mouse: [f32; 2],
    sound_system: SoundSystem
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

        Self {
            start: Instant::now(),
            display: Arc::new(display.clone()),
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            sound_system
        }
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        let time = context.start.elapsed().as_secs_f32();
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        let r = time.sin() * 0.5 + 0.5;
        let g = (time + 5.0).sin() * 0.5 + 0.5;
        let b = (time + 10.0).sin() * 0.5 + 0.5;

        let (x, y) = canvas.dimensions();

        let shader = canvas.shaders().borrow().default();
        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport())
        };
        let params = DrawParameters::default();

        canvas.rect([40.0, 20.0, 100.0, 20.0], [1.0, 0.0, 0.0, 1.0], &*shader, &uniforms, &params);

        canvas.text("Привет, мир!", x / 2.0, y - 50.0, &FontParameters {
            color: [r, g, b, 1.0],
            size: 72,
            align_horizontal: TextAlignHorizontal::Center,
            .. Default::default()
        });
    }

    fn on_resized(&mut self, context: &mut WindowContext, width: f32, height: f32) {
        context.width = width;
        context.height = height;
    }

    fn on_mouse_scroll(&mut self, context: &mut WindowContext, delta: MouseScrollDelta, modifiers: ModifiersState) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {

            }
            _ => {}
        }
    }

    fn on_mouse_button(&mut self, context: &mut WindowContext, state: ElementState, button: MouseButton, modifiers: ModifiersState) {
        if button == MouseButton::Left && state == ElementState::Pressed {

        }
    }

    fn on_mouse_move(&mut self, context: &mut WindowContext, x: f32, y: f32) {
        context.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, context: &mut WindowContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {
            if key == VirtualKeyCode::Back && input.state == ElementState::Pressed {

                let _ = context.sound_system.play_streaming_file("resources/sounds/laser.ogg")
                    .expect("Error playing sound");
                println!("pew-pew");
            }
        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(800, 600), 24, WindowHandler);
}