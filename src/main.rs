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
    fn restart(&mut self) {

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
    Flagged
}

impl WindowContext {

}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        // let time = context.start.elapsed().as_secs_f32();
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        let tiles = canvas.textures().try_borrow_mut().unwrap()
            .get_or_load(String::from("tiles"), "resources/textures/tiles.png")
            .unwrap();

        let (width, height) = canvas.dimensions();

        let program = canvas.shaders().borrow().default();

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

        let s = context.width / (GRID as f32);

        for x in 0..GRID {
            for y in 0..GRID {
                let cell = &context.grid[x][y];
                let x = x as f32 * s;
                let y = y as f32 * s;
                /*if cell.visibility == Visibility::Hidden {
                    canvas.rect([x, y, s, s], [1.0, 1.0, 1.0, 1.0], &program, &uniforms, &params);
                } else {
                    canvas.rect([x, y, s, s], [0.0, 0.0, 0.0, 1.0], &program, &uniforms, &params);
                }*/
                if cell.mine {
                    canvas.rect([x, y, s, s], [1.0, 0.0, 0.0, 1.0], &program, &uniforms, &params);
                } else {
                    canvas.text(&format!("{}", cell.counter), x + 15.0, y + 5.0, &FontParameters {
                        color: [1.0, 1.0, 1.0, 1.0],
                        size: 54,
                        align_horizontal: TextAlignHorizontal::Left,
                        .. Default::default()
                    });
                }
            }
        }


        if context.game_over {
            canvas.text("Вы проиграли", width / 2.0, height - 100.0, &FontParameters {
                color: [1.0, 0.0, 0.0, 1.0],
                size: 54,
                align_horizontal: TextAlignHorizontal::Center,
                .. Default::default()
            });
        }
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
        if button == MouseButton::Left && state == ElementState::Pressed {
            context.grid = [[Cell {
                mine: false,
                counter: 0,
                visibility: Visibility::Hidden
            }; GRID]; GRID];

            let [mx, my] = context.mouse;
            let x = (mx / context.width * GRID as f32) as usize;
            let y = (my / context.height * GRID as f32) as usize;
            context.grid[x][y].visibility = Visibility::Revealed;

            let mut mines = GRID + 1;

            while mines > 0 {
                let x = random::<usize>() % GRID;
                let y = random::<usize>() % GRID;

                let cell = &mut context.grid[x][y];

                if !cell.mine {
                    cell.mine = true;
                    mines -= 1;

                    for dx in -1..=1isize {
                        for dy in -1..=1isize {
                            let x = x as isize + dx;
                            let y = y as isize + dy;
                            if x >= 0 && y >= 0 && x < GRID as isize && y < GRID as isize {
                                context.grid[x as usize][y as usize].counter += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_mouse_move(&mut self, context: &mut WindowContext, x: f32, y: f32) {
        context.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, context: &mut WindowContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {

        }
    }
}

fn main() {
    window::create("Разработка игр", LogicalSize::new(400, 400), 24, WindowHandler);
}