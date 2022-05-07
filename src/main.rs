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

const GRID: usize = 11;
const GAME_SPEED: f32 = 4.0;

const EAT: &[u8] = include_bytes!("../resources/sounds/eat.ogg");

struct WindowContext {
    start: Instant,
    display: Arc<Display>,
    width: f32,
    height: f32,
    color: [f32; 3],
    mouse: [f32; 2],
    grid: [[Cell; GRID]; GRID],
    dir: Option<Dir>,
    key_dir: Option<Dir>,
    tail: VecDeque<[usize; 2]>,
    timer: f32,
    game_over: bool,
    sound_system: SoundSystem
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {

        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

        let mut grid = [[Cell::Air; GRID]; GRID];
        grid[GRID / 2][GRID / 2] = Cell::Head(Dir::Up);
        grid[0][0] = Cell::Apple;

        Self {
            start: Instant::now(),
            display: Arc::new(display.clone()),
            timer: 0.0,
            dir: None,
            key_dir: None,
            game_over: false,
            tail: VecDeque::new(),
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            grid,
            sound_system
        }
    }
}

impl WindowContext {
    fn restart(&mut self) {
        self.start = Instant::now();
        self.timer = 0.0;
        self.dir = None;
        self.key_dir = None;
        self.tail.clear();
        let mut grid = [[Cell::Air; GRID]; GRID];
        grid[GRID / 2][GRID / 2] = Cell::Head(Dir::Up);
        grid[0][0] = Cell::Apple;
        self.grid = grid;
        self.game_over = false;
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Dir {
    Up,
    Down,
    Left,
    Right
}

impl Dir {
    fn opposite(&self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Cell {
    Air,
    Apple,
    Head(Dir),
    Body(Dir, Dir)
}

impl WindowContext {
    pub fn tick(&mut self) {
        if let Some(dir) = self.dir {
            for x in 0..GRID {
                for y in 0..GRID {
                    let slot = self.grid[y][x];
                    if let Cell::Head(head_dir) = slot {
                        let old = self.move_to(x, y, head_dir, dir, Cell::Head(dir));
                        self.dir = self.key_dir;
                        self.tail.push_front([x, y]);
                        match old {
                            Cell::Apple => {
                                let _ = self.sound_system.play_streaming_bytes(&EAT)
                                    .expect("Error playing sound");
                                loop {
                                    let ax = rand::random::<usize>() % GRID;
                                    let ay = rand::random::<usize>() % GRID;
                                    let target = self.grid[ay][ax];
                                    if target == Cell::Air {
                                        self.grid[ay][ax] = Cell::Apple;
                                        break;
                                    }
                                }
                            }
                            Cell::Body(_, _) => {
                                self.game_over = true;
                            }
                            _ => {
                                if let Some([x, y]) = self.tail.pop_back() {
                                    self.grid[y][x] = Cell::Air;
                                }
                            }
                        }
                        return;
                    }
                }
            }
        } else { //Начало игры
            self.dir = self.key_dir;
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, head_dir: Dir, dir: Dir, slot: Cell) -> Cell {
        self.grid[y][x] = Cell::Body(head_dir.opposite(), dir);
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

        let tiles = canvas.textures().try_borrow_mut().unwrap()
            .get("textures/tiles");

        if !context.game_over {
            let last_second = context.timer as u32;

            context.timer += GAME_SPEED * time_elapsed;

            let current_second = context.timer as u32;

            if current_second > last_second {
                context.tick();
            }
        }

        let (x, y) = canvas.dimensions();

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

        let size = x / GRID as f32;

        for row in 0..GRID {
            for column in 0..GRID {
                let slot = context.grid[row][column];
                let color = [1.0; 4];
                let x = column as f32 * size;
                let y = row as f32 * size;
                let w = size;
                let h = size;

                let is_tail = if let Some(tail) = context.tail.back() {
                    tail == &[column, row]
                } else {
                    false
                };

                let slot = match slot {
                    Cell::Apple => 15,
                    Cell::Head(dir) => {
                        match dir {
                            Dir::Up => 3,
                            Dir::Down => 9,
                            Dir::Left => 8,
                            Dir::Right => 4
                        }
                    },
                    Cell::Body(from, to) => {
                        if is_tail {
                            match to {
                                Dir::Up => 13,
                                Dir::Down => 19,
                                Dir::Left => 18,
                                Dir::Right => 14
                            }
                        } else {
                            match from {
                                Dir::Down => {
                                    match to {
                                        Dir::Down => unreachable!(),
                                        Dir::Up => 7,
                                        Dir::Left => 2,
                                        Dir::Right => 0
                                    }
                                },
                                Dir::Up => {
                                    match to {
                                        Dir::Down => 7,
                                        Dir::Up => unreachable!(),
                                        Dir::Left => 12,
                                        Dir::Right => 5
                                    }
                                },
                                Dir::Left => {
                                    match to {
                                        Dir::Down => 2,
                                        Dir::Up => 12,
                                        Dir::Left => unreachable!(),
                                        Dir::Right => 1
                                    }
                                },
                                Dir::Right => {
                                    match to {
                                        Dir::Down => 0,
                                        Dir::Up => 5,
                                        Dir::Left => 1,
                                        Dir::Right => unreachable!()
                                    }
                                }
                            }
                        }
                    }
                    _ => 6
                };
                let texture_x = (slot % 5) as f32 / 5.0;
                let texture_y = (slot / 5) as f32 / 4.0;
                canvas.generic_shape(&PrimitiveType::TriangleFan, &[
                    Vertex::pos([x    , y    , 0.0]).color(color).uv([texture_x      , texture_y]),
                    Vertex::pos([x + w, y    , 0.0]).color(color).uv([texture_x + 0.2, texture_y]),
                    Vertex::pos([x + w, y + h, 0.0]).color(color).uv([texture_x + 0.2, texture_y + 0.25]),
                    Vertex::pos([x    , y + h, 0.0]).color(color).uv([texture_x      , texture_y + 0.25]),
                ], true, false, &*program, &uniforms, &params);
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
        canvas.text(format!("Счет: {}", context.tail.len()), x / 2.0, y - 50.0, &FontParameters {
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
                    context.restart();
                } else {
                    let new_dir = match key {
                        VirtualKeyCode::W => Some(Dir::Up),
                        VirtualKeyCode::S => Some(Dir::Down),
                        VirtualKeyCode::A => Some(Dir::Left),
                        VirtualKeyCode::D => Some(Dir::Right),
                        _ => None
                    };
                    if let Some(inertia) = context.dir {
                        if let Some(new_dir) = new_dir {
                            if new_dir != inertia.opposite() {
                                context.key_dir = Some(new_dir);
                            }
                        }
                    } else {
                        context.key_dir = new_dir;
                    }
                }
            }
        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(400, 400), 24, WindowHandler);
}