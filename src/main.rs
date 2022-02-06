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

const GRID: usize = 11;
const GAME_SPEED: f32 = 2.0;

struct WindowContext {
    start: Instant,
    display: Arc<Display>,
    width: f32,
    height: f32,
    color: [f32; 3],
    mouse: [f32; 2],
    grid: [[Cell; GRID]; GRID],
    dir: Option<Dir>,
    timer: f32,
    length: usize,
    game_over: bool,
    sound_system: SoundSystem
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

        let mut grid = [[Cell::Air; GRID]; GRID];

        grid[GRID / 2][GRID / 2] = Cell::Head;

        grid[0][0] = Cell::Apple;

        Self {
            start: Instant::now(),
            display: Arc::new(display.clone()),
            timer: 0.0,
            dir: None,
            length: 0,
            game_over: false,
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            grid,
            sound_system
        }
    }
}

#[derive(Copy, Clone)]
enum Dir {
    Up,
    Down,
    Left,
    Right
}

#[derive(Copy, Clone, PartialEq)]
enum Cell {
    Air,
    Apple,
    Head,
    Tail
}

impl WindowContext {
    pub fn tick(&mut self) {
        if let Some(dir) = self.dir {
            for x in 0..GRID {
                for y in 0..GRID {
                    let slot = self.grid[y][x];
                    if slot == Cell::Head {
                        let old = self.move_to(x, y, dir, Cell::Head);
                        if old == Cell::Apple {
                            self.length += 1;
                            println!("Съели яблоко");
                            let ax = rand::random::<usize>() % GRID;
                            let ay = rand::random::<usize>() % GRID;
                            self.grid[ay][ax] = Cell::Apple;
                            if self.length == 1 { // Хвоста еще не было
                                self.grid[y][x] = Cell::Tail;
                            } else {

                            }
                        } else if old == Cell::Tail {
                            self.game_over = true;
                        }
                        return;
                    }
                }
            }
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, dir: Dir, slot: Cell) -> Cell {
        self.grid[y][x] = Cell::Tail;
        let cell = match dir {
            Dir::Up => {
                if y == 0 {
                    &mut self.grid[GRID - 1][x]
                } else {
                    &mut self.grid[y - 1][x]
                }
            },
            Dir::Down => {
                if y + 1 == GRID {
                    &mut self.grid[0][x]
                } else {
                    &mut self.grid[y + 1][x]
                }
            },
            Dir::Left => {
                if x == 0 {
                    &mut self.grid[y][GRID - 1]
                } else {
                    &mut self.grid[y][x - 1]
                }
            },
            Dir::Right => {
                if x + 1 == GRID {
                    &mut self.grid[y][0]
                } else {
                    &mut self.grid[y][x + 1]
                }
            }
        };
        let old = *cell;
        *cell = slot;
        old
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        let time = context.start.elapsed().as_secs_f32();
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        if !context.game_over {
            let last_second = context.timer as u32;

            context.timer += GAME_SPEED * time_elapsed;

            let current_second = context.timer as u32;

            if current_second > last_second {
                context.tick();
            }
        }

        let (x, y) = canvas.dimensions();

        let shader = canvas.shaders().borrow().default();
        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport())
        };
        let params = DrawParameters::default();

        let size = x / GRID as f32;

        for row in 0..GRID {
            for column in 0..GRID {
                let slot = context.grid[row][column];
                let color = match slot {
                    Cell::Apple => [1.0, 0.0, 0.0, 1.0],
                    Cell::Head  => [0.0, 1.0, 0.0, 1.0],
                    Cell::Tail  => [0.0, 0.5, 0.0, 1.0],
                    _           => [0.0, 0.0, 0.0, 1.0],
                };
                let x = column as f32 * size;
                let y = row as f32 * size;
                canvas.rect([x, y, size, size], color, &*shader, &uniforms, &params);
            }
        }

        if context.game_over {
            canvas.text("Вы проиграли", x / 2.0, y - 100.0, &FontParameters {
                color: [1.0, 0.0, 0.0, 1.0],
                size: 54,
                align_horizontal: TextAlignHorizontal::Center,
                .. Default::default()
            });
        }
        canvas.text(format!("Счет: {}", context.length), x / 2.0, y - 50.0, &FontParameters {
            color: [1.0, 1.0, 1.0, 1.0],
            size: 54,
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
            if input.state == ElementState::Pressed {
                if key == VirtualKeyCode::Back {

                    let _ = context.sound_system.play_streaming_file("resources/sounds/laser.ogg")
                        .expect("Error playing sound");
                    println!("pew-pew");
                } else if key == VirtualKeyCode::W {
                    context.dir = Some(Dir::Up);
                } else if key == VirtualKeyCode::S {
                    context.dir = Some(Dir::Down);
                } else if key == VirtualKeyCode::A {
                    context.dir = Some(Dir::Left);
                } else if key == VirtualKeyCode::D {
                    context.dir = Some(Dir::Right);
                }
            }
        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(400, 400), 24, WindowHandler);
}