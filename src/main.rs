#![windows_subsystem="windows"]
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use cgmath::{Deg, Matrix4, SquareMatrix, vec3};
use glium::{Blend, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, uniform, VertexBuffer};
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, StartCause, VirtualKeyCode};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::texture::SrgbTexture2d;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use rand::random;
use crate::audio::SoundSystem;
use crate::font::{FontParameters, TextAlignHorizontal};
use crate::render::{Canvas, Vertex};

use crate::window::{Context, Handler};

#[macro_use]
extern crate glium;

mod window;
mod shaders;
mod textures;
mod render;
mod font;
mod audio;

const GRID: usize = 9;
const S: u32 = 32;
const W: u32 = S * GRID as u32;
const H: u32 = S * GRID as u32;

const EAT: &[u8] = include_bytes!("../resources/sounds/eat.ogg");

struct WindowContext {
    start: Option<Instant>,
    display: Arc<Display>,
    width: f32,
    height: f32,
    mouse: [f32; 2],
    grid: [[Cell; GRID]; GRID],
    game_over: bool,
    sound_system: SoundSystem
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

        let cell = Cell {
            mine: false,
            counter: 0,
            visibility: Visibility::Hidden
        };

        let mut grid = [[cell; GRID]; GRID];

        Self {
            start: None,
            display: Arc::new(display.clone()),
            game_over: false,
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            grid,
            sound_system
        }
    }
}

impl WindowContext {
    fn reset(&mut self, click_x: usize, click_y: usize, keep_flags: bool) {
        self.start = Some(Instant::now());

        for x in 0..GRID {
            for y in 0..GRID {
                let cell = &mut self.grid[x][y];
                *cell = Cell {
                    mine: false,
                    counter: 0,
                    visibility: if keep_flags { cell.visibility } else { Visibility::Hidden }
                };
            }
        }

        let mut mines = GRID + 1;

        while mines > 0 {
            let x = random::<usize>() % GRID;
            let y = random::<usize>() % GRID;

            if x == click_x && y == click_y {
                continue;
            }

            let cell = &mut self.grid[x][y];

            if !cell.mine {
                cell.mine = true;
                mines -= 1;

                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let x = x as i32 + dx;
                        let y = y as i32 + dy;
                        if x >= 0 && y >= 0 && x < GRID as i32 && y < GRID as i32 {
                            self.grid[x as usize][y as usize].counter += 1;
                        }
                    }
                }
            }
        }
    }

    fn reveal(&mut self, cell_x: usize, cell_y: usize, visited: &mut Vec<(usize, usize)>) {
        visited.push((cell_x, cell_y));

        let cell = &mut self.grid[cell_x][cell_y];

        if cell.visibility != Visibility::Hidden {
            return;
        }

        cell.visibility = Visibility::Revealed;
        let counter = cell.counter;

        if cell.mine {
            self.game_over = true;
            for x in 0..GRID {
                for y in 0..GRID {
                    let mut cell = &mut self.grid[x][y];
                    cell.visibility = match cell.visibility {
                        Visibility::Hidden => if cell.mine { Visibility::Revealed } else { Visibility::Hidden },
                        Visibility::Revealed => Visibility::Revealed,
                        Visibility::Flagged => if cell.mine { Visibility::Flagged } else { Visibility::Wrong },
                        Visibility::Wrong => Visibility::Wrong,
                        Visibility::Question => Visibility::Question
                    }
                }
            }
        } else if counter == 0 {
            for dx in -1..=1i32 {
                for dy in -1..=1i32 {
                    let x = cell_x as i32 + dx;
                    let y = cell_y as i32 + dy;
                    if x >= 0 && y >= 0 && x < GRID as i32 && y < GRID as i32 {
                        let x = x as usize;
                        let y = y as usize;
                        if !visited.contains(&(x, y)) {
                            let neighbour = &self.grid[x][y];
                            if !neighbour.mine && (neighbour.counter != 0 || dx.abs() != dy.abs()) {
                                self.reveal(x, y, visited);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Cell {
    mine: bool,
    counter: u8,
    visibility: Visibility
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Visibility {
    Hidden,
    Revealed,
    Flagged,
    Wrong,
    Question
}

impl WindowContext {

}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        // let time = context.start.elapsed().as_secs_f32();
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        let tiles = canvas.textures().try_borrow_mut().unwrap()
            .tiles();

        let (width, height) = canvas.dimensions();

        let program = canvas.shaders().borrow().textured();

        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport()),
            tex: tiles.sampled()
                .anisotropy(4)
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest)
        };
        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        let s = S as f32;

        let color = [1.0;4];

        for x in 0..GRID {
            for y in 0..GRID {
                let cell = &context.grid[x][y];
                let x = x as f32 * s;
                let y = y as f32 * s;
                let slot = match cell.visibility {
                    Visibility::Hidden => 9,
                    Visibility::Flagged => 10,
                    Visibility::Wrong => 11,
                    Visibility::Question => 12,
                    Visibility::Revealed => {
                        if cell.mine {
                            13
                        } else {
                            cell.counter
                        }
                    }
                };

                let texture_x = (slot % 5) as f32 / 5.0;
                let texture_y = (slot / 5) as f32 / 4.0;
                canvas.generic_shape(&PrimitiveType::TriangleFan, &[
                    Vertex::pos([x    , y    , 0.0]).color(color).uv([texture_x      , texture_y]),
                    Vertex::pos([x + s, y    , 0.0]).color(color).uv([texture_x + 0.2, texture_y]),
                    Vertex::pos([x + s, y + s, 0.0]).color(color).uv([texture_x + 0.2, texture_y + 1.0 / 4.0]),
                    Vertex::pos([x    , y + s, 0.0]).color(color).uv([texture_x      , texture_y + 1.0 / 4.0]),
                ], true, false, &*program, &uniforms, &params);
            }
        }

/*
        if context.game_over {
            canvas.text("Вы проиграли", width / 2.0, height - 100.0, &FontParameters {
                color: [1.0, 0.0, 0.0, 1.0],
                size: 54,
                align_horizontal: TextAlignHorizontal::Center,
                .. Default::default()
            });
        }*/
        /*let [mx, my] = context.mouse;
        let x = (mx / context.width * GRID as f32) as usize;
        let y = (my / context.height * GRID as f32) as usize;
        canvas.text(format!("Мышь: {}, {}", x, y), width / 2.0, height - 50.0, &FontParameters {
            color: [1.0, 1.0, 1.0, 1.0],
            size: 54,
            align_horizontal: TextAlignHorizontal::Center,
            .. Default::default()
        });*/
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
        if context.game_over {
            return;
        }
        let [mx, my] = context.mouse;
        let x = (mx / context.width * GRID as f32) as usize;
        let y = (my / context.height * GRID as f32) as usize;
        if button == MouseButton::Left && state == ElementState::Pressed {
            if context.start.is_none() {
                context.reset(x, y, true);
            }
            let mut visited = Vec::new();
            context.reveal(x, y, &mut visited);
        }
        if button == MouseButton::Right && state == ElementState::Pressed {
            let cell = &mut context.grid[x][y];
            cell.visibility = match cell.visibility {
                Visibility::Revealed => Visibility::Revealed,
                Visibility::Wrong => Visibility::Wrong,
                Visibility::Hidden => Visibility::Flagged,
                Visibility::Flagged => Visibility::Question,
                Visibility::Question => Visibility::Hidden
            };
        }
    }

    fn on_mouse_move(&mut self, context: &mut WindowContext, x: f32, y: f32) {
        context.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, context: &mut WindowContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {
            if key == VirtualKeyCode::Back && input.state == ElementState::Pressed {
                context.game_over = false;
                context.reset(usize::MAX, usize::MAX, false);
            }
        }
    }
}

fn main() {
    window::create("Сапёр", LogicalSize::new(W, H), 24, WindowHandler);
}