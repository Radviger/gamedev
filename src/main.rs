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

const CELLS: usize = 3;

struct Brick {
    bounds: [f32; 4],
    color: [f32; 4]
}

struct WindowContext {
    start: Instant,
    display: Arc<Display>,
    width: f32,
    height: f32,
    color: [f32; 3],
    mouse: [f32; 2],
    cells: [[u8; CELLS]; CELLS],
    winner: u8
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        Self {
            start: Instant::now(),
            display: Arc::new(display.clone()),
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            cells: [[0; CELLS]; CELLS],
            winner: 0
        }
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        let time = context.start.elapsed().as_secs_f32();
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        let (width, height) = canvas.dimensions();

        let shader = canvas.shaders().borrow().default();
        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport())
        };
        let params = DrawParameters::default();

        let cw = width / CELLS as f32;
        let ch = height / CELLS as f32;

        let size = (width / CELLS as f32) as u32;

        if context.winner == 0 {
            for (y, line) in context.cells.iter().enumerate() {
                for (x, value) in line.iter().enumerate() {
                    let (letter, color) = match value {
                        1 => ("x", [1.0, 0.0, 0.0, 1.0]),
                        2 => ("o", [0.0, 0.0, 1.0, 1.0]),
                        _ => ("?", [1.0, 1.0, 1.0, 1.0])
                    };
                    canvas.text(letter, x as f32 * cw + cw / 2.0, y as f32 * ch + (size as f32 / 3.0), &FontParameters {
                        color,
                        size,
                        align_horizontal: TextAlignHorizontal::Center,
                        ..Default::default()
                    });
                }
            }
        } else {
            let (winner, color) = match context.winner {
                1 => ("Победили крестики", [1.0, 0.0, 0.0, 1.0]),
                2 => ("Победили нолики", [0.0, 0.0, 1.0, 1.0]),
                _ => ("Ничья", [1.0, 1.0, 1.0, 1.0])
            };
            canvas.text(winner, width / 2.0, height - 50.0, &FontParameters {
                color,
                size: 72,
                align_horizontal: TextAlignHorizontal::Center,
                .. Default::default()
            });
        }
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
        let cw = context.width / CELLS as f32;
        let ch = context.height / CELLS as f32;
        let [mouse_x, mouse_y] = context.mouse;
        let y = (mouse_y / ch) as usize;
        let x = (mouse_x / cw) as usize;

        if state == ElementState::Pressed {
            match context.cells[y][x] {
                1 | 2 => {},
                _ => {
                    if button == MouseButton::Left {
                        context.cells[y][x] = 1;
                    }
                    if button == MouseButton::Right {
                        context.cells[y][x] = 2;
                    }
                }
            }
        }

        // Horizontal strike

        for y in 0..CELLS {
            let mut counter = [0, 0, 0];
            for x in 0..CELLS {
                match context.cells[y][x] {
                    1 => {
                        counter[1] += 1;
                    },
                    2 => {
                        counter[2] += 1;
                    },
                    _ => {
                        counter[0] += 1;
                    }
                }
            }
            if counter[1] == CELLS { //x wins
                context.winner = 1;
                return;
            }
            if counter[2] == CELLS { //o wins
                context.winner = 2;
                return;
            }
        }

        // Vertical strike

        for x in 0..CELLS {
            let mut counter = [0, 0, 0];
            for y in 0..CELLS {
                match context.cells[y][x] {
                    1 => {
                        counter[1] += 1;
                    },
                    2 => {
                        counter[2] += 1;
                    },
                    _ => {
                        counter[0] += 1;
                    }
                }
            }
            if counter[1] == CELLS { //x wins
                context.winner = 1;
                return;
            }
            if counter[2] == CELLS { //o wins
                context.winner = 2;
                return;
            }
        }

        // First diagonal

        {
            let mut counter = [0, 0, 0];
            for d in 0..CELLS {
                match context.cells[d][d] {
                    1 => {
                        counter[1] += 1;
                    },
                    2 => {
                        counter[2] += 1;
                    },
                    _ => {
                        counter[0] += 1;
                    }
                }
            }
            if counter[1] == CELLS { //x wins
                context.winner = 1;
                return;
            }
            if counter[2] == CELLS { //o wins
                context.winner = 2;
                return;
            }
        }

        //Second diagonal

        {
            let mut counter = [0, 0, 0];
            for d in 0..CELLS {
                match context.cells[d][CELLS-d-1] {
                    1 => {
                        counter[1] += 1;
                    },
                    2 => {
                        counter[2] += 1;
                    },
                    _ => {
                        counter[0] += 1;
                    }
                }
            }
            if counter[1] == CELLS { //x wins
                context.winner = 1;
                return;
            }
            if counter[2] == CELLS { //o wins
                context.winner = 2;
                return;
            }
        }

    }

    fn on_mouse_move(&mut self, context: &mut WindowContext, x: f32, y: f32) {
        context.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, context: &mut WindowContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {
            if key == VirtualKeyCode::Back && input.state == ElementState::Pressed {
                context.winner = 0;
                context.cells = [[0; CELLS]; CELLS];
            }
        }
    }
}

fn main() {
    window::create("Крестики-нолики", LogicalSize::new(480, 480), 24, WindowHandler);
}