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

const GRID: usize = 10;
const S: u32 = 32;
const W: u32 = 2 * S * GRID as u32;
const H: u32 = S * GRID as u32;
const EAT: &[u8] = include_bytes!("../resources/sounds/eat.ogg");

struct GameContext {
    start: Option<Instant>,
    display: Arc<Display>,
    width: f32,
    height: f32,
    mouse: [f32; 2],
    our_field: Field,
    enemy_field: Field,
    inventory: [u8; 4],
    game_over: bool,
    sound_system: SoundSystem,
    length: u8,
    dir: Dir
}

struct Field {
    cells: [[Cell; GRID]; GRID]
}

impl Field {
    fn new() -> Field {
        Field {
            cells: [[Cell { ship: None }; GRID]; GRID]
        }
    }

    fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[x][y]
    }

    fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[x][y] = cell;
    }

    fn has_collision(&self, x: usize, y: usize, length: u8, dir: Dir) -> bool {
        for i in 0..length {
            let (dx, dy) = dir.to_vec();
            let x = x as isize + dx * i as isize;
            let y = y as isize + dy * i as isize;
            if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                for ddx in -1..=1 {
                    for ddy in -1..=1 {
                        let x = x as i32 + ddx;
                        let y = y as i32 + ddy;
                        if x >= 0 && y >= 0 && x < GRID as i32 && y < GRID as i32 {
                            let cell = &self.cells[x as usize][y as usize];
                            if cell.ship.is_some() {
                                return true;
                            }
                        }
                    }
                }
            } else {
                return true;
            }
        }
        return false;
    }
}

impl Context for GameContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

        let cell = Cell {
            ship: None
        };


        let mut our_field = Field::new();
        let mut enemy_field = Field::new();

        Self {
            start: None,
            display: Arc::new(display.clone()),
            game_over: false,
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            our_field,
            enemy_field,
            sound_system,
            inventory: [4, 3, 2, 1],
            length: 4,
            dir: Dir::Down,
        }
    }
}

impl GameContext {
    fn reset(&mut self, click_x: usize, click_y: usize, keep_flags: bool) {

    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
struct Cell {
    ship: Option<(Dir, u8)>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn to_vec(&self) -> (isize, isize) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0)
        }
    }
}

impl GameContext {
    fn has_selected_ship_model(&self) -> bool {
        self.inventory[self.length as usize - 1] > 0
    }
}

struct WindowHandler;

impl Handler<GameContext> for WindowHandler {
    fn draw_frame(&mut self, game: &mut GameContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        let s = S as f32;

        let shader = canvas.shaders().borrow().default();
        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport())
        };
        let params = DrawParameters::default();

        for x in 0..GRID {
            for y in 0..GRID {
                let cell = game.our_field.get(x, y);
                let x = x as f32 * s;
                let y = y as f32 * s;
                if let Some((dir, i)) = cell.ship {
                    canvas.rect([x, y, s, s], [1.0, 1.0, 1.0, 1.0], &*shader, &uniforms, &params);
                    canvas.text(&format!("{i}"), x + s / 2.0, y + s / 3.0, &FontParameters {
                        size: 52,
                        color: [1.0; 4],
                        .. Default::default()
                    });
                }
            }
        }
        for x in 0..GRID {
            for y in 0..GRID {
                let cell = game.enemy_field.get(x, y);
                let x = x as f32 * s;
                let y = y as f32 * s;
                if let Some((dir, i)) = cell.ship {
                    canvas.rect([x + game.width / 2.0, y, s, s], [1.0, 1.0, 1.0, 1.0], &*shader, &uniforms, &params);
                    canvas.text(&format!("{i}"), x + game.width / 2.0 + s / 2.0 - 3.0, y + s / 3.0 - 3.0, &FontParameters {
                        size: 52,
                        color: [1.0, 0.0, 1.0, 1.0],
                        .. Default::default()
                    });
                }
            }
        }

        let [mx, my] = game.mouse;
        let x = (mx / (game.width / 2.0) * GRID as f32) as usize;
        let y = (my / game.height * GRID as f32) as usize;

        if mx < game.width / 2.0 {
            if game.start.is_none() {
                let mut error = !game.has_selected_ship_model() || game.our_field.has_collision(x, y, game.length, game.dir);

                let color = if !error {
                    [0.0, 1.0, 1.0, 1.0]
                } else {
                    [1.0, 0.0, 0.0, 1.0]
                };
                for i in 0..game.length {
                    let (dx, dy) = game.dir.to_vec();
                    let x = x as isize + dx * i as isize;
                    let y = y as isize + dy * i as isize;
                    if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                        canvas.rect([x as f32 * s, y as f32 * s, s, s], color, &*shader, &uniforms, &params);
                    }
                }
            }
        } else if game.start.is_some() {
            let color = [0.0, 1.0, 0.0, 1.0];
            canvas.rect([x as f32 * s, y as f32 * s, s, s], color, &*shader, &uniforms, &params);
        }

        canvas.rect([game.width / 2.0 - 1.0, 0.0, 2.0, game.height], [1.0; 4], &shader, &uniforms, &params);
    }

    fn on_mouse_button(&mut self, game: &mut GameContext, state: ElementState, button: MouseButton, modifiers: ModifiersState) {
        let [mx, my] = game.mouse;
        let x = (mx / (game.width / 2.0) * GRID as f32) as usize;
        let y = (my / game.height * GRID as f32) as usize;

        if mx <= game.width / 2.0 {
            if button == MouseButton::Left && state == ElementState::Pressed {
                let mut error = !game.has_selected_ship_model() || game.our_field.has_collision(x, y, game.length, game.dir);

                if !error {
                    for i in 0..game.length {
                        let (dx, dy) = game.dir.to_vec();
                        let x = x as isize + dx * i as isize;
                        let y = y as isize + dy * i as isize;
                        if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                            game.our_field.set(x as usize, y as usize, Cell { ship: Some((game.dir, i)) });
                        }
                    }
                    game.inventory[game.length as usize - 1] -= 1;

                    if game.inventory[0] == 0
                        && game.inventory[1] == 0
                        && game.inventory[2] == 0
                        && game.inventory[3] == 0 {

                        game.start = Some(Instant::now());

                        let mut inventory = [4, 3, 2, 1];

                        for length in 1..=4 {
                            while inventory[length as usize - 1] > 0 {
                                let x = random::<usize>() % GRID;
                                let y = random::<usize>() % GRID;
                                let dir = match random::<u8>() % 4 {
                                    0 => Dir::Up,
                                    1 => Dir::Right,
                                    2 => Dir::Down,
                                    3 => Dir::Left,
                                    other => unreachable!("Unknown random direction {other}")
                                };

                                if !game.enemy_field.has_collision(x, y, length, dir) {
                                    for i in 0..length {
                                        let (dx, dy) = dir.to_vec();
                                        let x = x as isize + dx * i as isize;
                                        let y = y as isize + dy * i as isize;
                                        if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                                            game.enemy_field.set(x as usize, y as usize, Cell { ship: Some((dir, i)) });
                                        }
                                    }
                                    inventory[length as usize - 1] -= 1;
                                }
                            }
                        }
                    }
                }
            }
        } else if game.start.is_some() {
            let x = x - 10;
            println!("Clicked enemy field at {x} {y}")
        }
    }

    fn on_mouse_move(&mut self, game: &mut GameContext, x: f32, y: f32) {
        game.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, game: &mut GameContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {
            if key == VirtualKeyCode::Key1 && input.state == ElementState::Pressed {
                game.length = 1;
            }
            if key == VirtualKeyCode::Key2 && input.state == ElementState::Pressed {
                game.length = 2;
            }
            if key == VirtualKeyCode::Key3 && input.state == ElementState::Pressed {
                game.length = 3;
            }
            if key == VirtualKeyCode::Key4 && input.state == ElementState::Pressed {
                game.length = 4;
            }
            if key == VirtualKeyCode::W || key == VirtualKeyCode::Up && input.state == ElementState::Pressed {
                game.dir = Dir::Up;
            }
            if key == VirtualKeyCode::A || key == VirtualKeyCode::Left && input.state == ElementState::Pressed {
                game.dir = Dir::Left;
            }
            if key == VirtualKeyCode::S || key == VirtualKeyCode::Down && input.state == ElementState::Pressed {
                game.dir = Dir::Down;
            }
            if key == VirtualKeyCode::D || key == VirtualKeyCode::Right && input.state == ElementState::Pressed {
                game.dir = Dir::Right;
            }
        }
    }

    fn on_mouse_scroll(&mut self, game: &mut GameContext, delta: MouseScrollDelta, modifiers: ModifiersState) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                game.length = ((game.length as f32 + y) as u8).clamp(1, 4)
            }
            _ => {}
        }
    }
}


fn main() {
    window::create("Морской бой", LogicalSize::new(W, H), 24, WindowHandler);
}