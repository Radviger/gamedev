use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::io::Cursor;
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
const W: u32 = 2 * S * GRID as u32 + 4 * S;
const H: u32 = S * GRID as u32 + 2 * S;
const MISS: &[u8] = include_bytes!("../resources/sounds/miss.ogg");
const HIT: &[u8] = include_bytes!("../resources/sounds/hit.ogg");
const CLICK: &[u8] = include_bytes!("../resources/sounds/click.ogg");
const SELECT: &[u8] = include_bytes!("../resources/sounds/select.ogg");

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
    dir: Dir,
    timer: f32
}

struct Field {
    cells: [[Cell; GRID]; GRID]
}


enum GridSelection {
    Placement {
        dir: Dir,
        length: u8,
        has_ship: bool
    },
    Shoot,
    None
}

impl Field {
    fn new() -> Field {
        Field {
            cells: [[Cell::Water; GRID]; GRID]
        }
    }

    fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[x][y]
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[x][y]
    }

    fn checked_get(&self, x: isize, y: isize) -> Option<&Cell> {
        if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
            Some(self.get(x as usize, y as usize))
        } else {
            None
        }
    }

    fn modify_all<M>(&mut self, x: usize, y: usize, start_len: i8, end_len: i8, dir: Dir, modifier: M) where M: Fn(&mut Cell, i8) {
        for i in start_len..end_len {
            let (dx, dy) = dir.to_vec();
            let x = x as isize + dx * i as isize;
            let y = y as isize + dy * i as isize;
            if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                modifier(&mut self.cells[x as usize][y as usize], i);
            }
        }
    }

    fn check_and_modify_all<C, M>(&mut self, x: usize, y: usize, start_len: i8, end_len: i8, dir: Dir, check: C, modifier: M)
        where
            C: Fn(&Cell) -> bool,
            M: Fn(&mut Cell, i8)
    {

        let mut success = true;
        let (dx, dy) = dir.to_vec();

        for i in start_len..end_len {
            let x = x as isize + dx * i as isize;
            let y = y as isize + dy * i as isize;
            if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                let cell = &self.cells[x as usize][y as usize];
                if !check(cell) {
                    success = false;
                    break;
                }
            } else {
                break;
            }
        }

        if success {
            for i in start_len..end_len {
                let x = x as isize + dx * i as isize;
                let y = y as isize + dy * i as isize;
                if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                    let cell = &mut self.cells[x as usize][y as usize];
                    if check(cell) {
                        modifier(cell, i);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

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
                            if let Cell::Ship { .. } = cell {
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

    fn draw(&self, x: f32, y: f32, canvas: &mut Canvas<Frame>, time: f32, mouse: [f32; 2], selection: GridSelection, hidden: bool) {
        let s = S as f32;
        let field_size = GRID as f32 * s;

        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport()),
            time: time
        };
        let params = DrawParameters::default();
        let shader = canvas.shaders().borrow().default();

        let font = FontParameters {
            size: 52,
            color: [1.0;4],
            .. Default::default()
        };

        for i in 0..10 {
            let y = y + i as f32 * s;
            canvas.text(&format!("{i}"), x - s + s / 2.0 - 1.0, y + s / 3.0 - 5.0, &font);
        }

        for c in 'A'..='J' {
            let i = c as u32 - 'A' as u32;
            let x = x + i as f32 * s;
            canvas.text(&format!("{c}"), x + s / 2.0 - 1.0, y + field_size + s / 3.0 - 5.0, &font);
        }

        for grid_x in 0..GRID {
            for grid_y in 0..GRID {
                let cell = self.get(grid_x, grid_y);
                let x = x + grid_x as f32 * s;
                let y = y + grid_y as f32 * s;

                match cell {
                    Cell::Water => {},
                    Cell::Miss => {
                        canvas.text("O", x + s / 2.0, y + s / 3.0, &font);
                    },
                    Cell::Ship { dir, length, fire, destroyed } => {
                        if *fire {
                            if *destroyed {
                                canvas.rect([x, y, s, s], [1.0, 0.0, 0.0, 1.0], &*shader, &uniforms, &params);
                            } else {
                                canvas.rect([x, y, s, s], [1.0, 1.0, 0.0, 1.0], &*shader, &uniforms, &params);
                            }
                        } else if !hidden {
                            canvas.rect([x, y, s, s], [1.0, 1.0, 1.0, 1.0], &*shader, &uniforms, &params);
                            /*canvas.text(&format!("{length}"), x + s / 2.0 - 3.0, y + s / 3.0 - 5.0, &FontParameters {
                                size: 52,
                                color: [0.0, 0.0, 0.0, 1.0],
                                .. Default::default()
                            });*/
                        }
                    }
                }
            }
        }

        let [mx, my] = mouse;
        if mx >= x && mx < x + field_size && my >= y && my < y + field_size {
            let grid_x = ((mx - x) / field_size * GRID as f32) as usize;
            let grid_y = ((my - y) / field_size * GRID as f32) as usize;

            match selection {
                GridSelection::Placement { dir, length, has_ship } => {
                    let error = !has_ship || self.has_collision(grid_x, grid_y, length, dir);
                    let color = if !error {
                        [0.0, 1.0, 1.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0, 1.0]
                    };
                    for i in 0..length {
                        let (dx, dy) = dir.to_vec();
                        let sub_x = grid_x as isize + dx * i as isize;
                        let sub_y = grid_y as isize + dy * i as isize;
                        if sub_x >= 0 && sub_y >= 0 && sub_x < GRID as isize && sub_y < GRID as isize {
                            canvas.rect([x + sub_x as f32 * s, y + sub_y as f32 * s, s, s], color, &*shader, &uniforms, &params);
                        }
                    }
                }
                GridSelection::Shoot => {
                    let color = [0.0, 1.0, 0.0, 1.0];
                    canvas.rect([x + grid_x as f32 * s, y + grid_y as f32 * s, s, s], color, &*shader, &uniforms, &params);
                }
                GridSelection::None => {}
            }

        }

        for grid_x in 0..=GRID {
            let x = x + grid_x as f32 * s;
            canvas.line([x, y], [x, y + field_size], [1.0; 4], &*shader, &uniforms, &params);
        }
        for grid_y in 0..=GRID {
            let y = y + grid_y as f32 * s;
            canvas.line([x, y], [x + field_size, y], [1.0; 4], &*shader, &uniforms, &params);
        }
    }
}

impl Context for GameContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

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
            timer: 0.0
        }
    }
}

impl GameContext {
    fn reset(&mut self, click_x: usize, click_y: usize, keep_flags: bool) {

    }

    fn play_sound(&mut self, sound: &[u8]) {
        if let Err(e) = self.sound_system.play_streaming_bytes(sound) {
            eprintln!("Sound system error: {:?}", e)
        }
    }

    fn get_grid_coordinates(&self, x: f32, y: f32, w: f32, h: f32) -> Option<[usize; 2]> {
        let [mx, my] = self.mouse;
        if mx >= x && mx < x + w && my >= y && my < y + h {
            Some([
                ((mx - x) / w * GRID as f32) as usize,
                ((my - y) / h * GRID as f32) as usize
            ])
        } else {
            None
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
enum Cell {
    Water,
    Miss,
    Ship {
        dir: Dir,
        length: u8,
        fire: bool,
        destroyed: bool
    }
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

    fn clockwise(&self) -> Self {
        match self {
            Dir::Up => Dir::Right,
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up
        }
    }

    fn counter_clockwise(&self) -> Self {
        match self {
            Dir::Up => Dir::Left,
            Dir::Left => Dir::Down,
            Dir::Down => Dir::Right,
            Dir::Right => Dir::Up
        }
    }

    fn opposite(&self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Left => Dir::Right,
            Dir::Down => Dir::Up,
            Dir::Right => Dir::Left
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

        game.timer += time_elapsed;

        let s = S as f32;

        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport()),
            time: game.timer
        };
        let params = DrawParameters::default();
        let shader = canvas.shaders().borrow().water();
        canvas.textured_rect([0.0, 0.0, game.width, game.height], [1.0; 4], &shader, &uniforms, &params);

        let game_started = game.start.is_some();
        let has_ship = game.has_selected_ship_model();

        game.our_field.draw(s, s, canvas, game.timer, game.mouse, if !game_started {
            GridSelection::Placement {
                dir: game.dir,
                length: game.length,
                has_ship
            }
        } else {
            GridSelection::None
        }, false);

        game.enemy_field.draw(game.width / 2.0 + s, s, canvas, game.timer, game.mouse, if game_started {
            GridSelection::Shoot
        } else {
            GridSelection::None
        }, true);
    }

    fn on_mouse_button(&mut self, game: &mut GameContext, state: ElementState, button: MouseButton, modifiers: ModifiersState) {
        let s = S as f32;
        let field_size = GRID as f32 * s;

        if let Some([x, y]) = game.get_grid_coordinates(s, s, field_size, field_size) {
            if button == MouseButton::Left && state == ElementState::Pressed {
                let mut error = !game.has_selected_ship_model() || game.our_field.has_collision(x, y, game.length, game.dir);

                if !error {
                    for i in 0..game.length {
                        let (dx, dy) = game.dir.to_vec();
                        let x = x as isize + dx * i as isize;
                        let y = y as isize + dy * i as isize;
                        if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                            game.our_field.set(x as usize, y as usize, Cell::Ship { dir: game.dir, length: i, fire: false, destroyed: false });
                        }
                    }
                    game.inventory[game.length as usize - 1] -= 1;

                    game.play_sound(&CLICK);

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
                                    game.enemy_field.modify_all(x, y, 0, length as i8, dir, |cell, i| {
                                        *cell = Cell::Ship { dir, length: i as u8, fire: false, destroyed: false };
                                    });
                                    inventory[length as usize - 1] -= 1;
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some([x, y]) = game.get_grid_coordinates(game.width / 2.0 + s, s, field_size, field_size) {
            if game.start.is_some() {
                let cell = game.enemy_field.get_mut(x, y);

                match cell {
                    Cell::Water => {
                        println!("Missed!");
                        game.play_sound(&MISS);
                        game.enemy_field.set(x, y, Cell::Miss)
                    },
                    Cell::Ship { fire, dir, length, destroyed } if !*fire => {
                        println!("Direct hit!");

                        *fire = true;

                        fn is_ship_burning(cell: &Cell) -> bool {
                            if let Cell::Ship { fire, .. } = cell {
                                *fire
                            } else {
                                false
                            }
                        }

                        fn destroy_ship(cell: &mut Cell, i: i8) {
                            if let Cell::Ship { destroyed, .. } = cell {
                                *destroyed = true;
                            }
                        }

                        let dir = *dir;
                        let length = *length as i8;
                        game.enemy_field.check_and_modify_all(x, y, -length, 4 - length, dir, is_ship_burning, destroy_ship);

                        game.play_sound(&HIT);
                    },
                    _ => {}
                }
            }
        }
    }

    fn on_mouse_move(&mut self, game: &mut GameContext, x: f32, y: f32) {
        game.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, game: &mut GameContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {
            let mut click = false;
            if key == VirtualKeyCode::Key1 && input.state == ElementState::Pressed {
                game.length = 1;
                click = true;
            }
            if key == VirtualKeyCode::Key2 && input.state == ElementState::Pressed {
                game.length = 2;
                click = true;
            }
            if key == VirtualKeyCode::Key3 && input.state == ElementState::Pressed {
                game.length = 3;
                click = true;
            }
            if key == VirtualKeyCode::Key4 && input.state == ElementState::Pressed {
                game.length = 4;
                click = true;
            }
            if key == VirtualKeyCode::W || key == VirtualKeyCode::Up && input.state == ElementState::Pressed {
                game.dir = Dir::Up;
                click = true;
            }
            if key == VirtualKeyCode::A || key == VirtualKeyCode::Left && input.state == ElementState::Pressed {
                game.dir = Dir::Left;
                click = true;
            }
            if key == VirtualKeyCode::S || key == VirtualKeyCode::Down && input.state == ElementState::Pressed {
                game.dir = Dir::Down;
                click = true;
            }
            if key == VirtualKeyCode::D || key == VirtualKeyCode::Right && input.state == ElementState::Pressed {
                game.dir = Dir::Right;
                click = true;
            }
            if click {
                game.play_sound(&SELECT);
            }
        }
    }

    fn on_mouse_scroll(&mut self, game: &mut GameContext, delta: MouseScrollDelta, modifiers: ModifiersState) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if modifiers.shift() {
                    if y > 0.0 {
                        game.dir = game.dir.clockwise();
                    } else {
                        game.dir = game.dir.counter_clockwise();
                    }
                } else {
                    game.length = ((game.length as f32 + y) as u8).clamp(1, 4)
                }
                game.play_sound(&SELECT);
            }
            _ => {}
        }
    }
}


fn main() {
    window::create("Морской бой", LogicalSize::new(W, H), 24, WindowHandler);
}