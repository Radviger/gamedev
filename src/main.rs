#![windows_subsystem="windows"]
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::iter::once;
use std::sync::Arc;
use std::time::Instant;

use cgmath::{Deg, Matrix4, point3, SquareMatrix, Transform, vec2, vec3, Vector2};
use glium::{Blend, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, Program, StencilOperation, StencilTest, Surface, Texture2d, uniform, VertexBuffer};
use glium::draw_parameters::Stencil;
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

const GRID: usize = 40;
const S: u32 = 16;
const W: u32 = S * GRID as u32;
const H: u32 = S * GRID as u32;

const EAT: &[u8] = include_bytes!("../resources/sounds/eat.ogg");

struct Light {
    pos: Vector2<f32>,
    color: [f32; 3],
    radius: f32
}

struct WindowContext {
    start: Option<Instant>,
    display: Arc<Display>,
    width: f32,
    height: f32,
    mouse: [f32; 2],
    grid: [[Cell; GRID]; GRID],
    color: [f32; 3],
    radius: f32,
    lights: Vec<Light>,
    game_over: bool,
    sound_system: SoundSystem
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);

        let sound_system = audio::SoundSystem::new().expect("Could not initialize audio device");

        let cell = Cell {
            block: false
        };

        let mut grid = [[cell; GRID]; GRID];

        Self {
            start: None,
            display: Arc::new(display.clone()),
            game_over: false,
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            radius: 10.0,
            color: [1.0; 3],
            lights: Vec::new(),
            grid,
            sound_system
        }
    }
}

impl WindowContext {
    fn reset(&mut self, click_x: usize, click_y: usize, keep_flags: bool) {
        self.start = Some(Instant::now());
    }
}

#[derive(Copy, Clone, Debug)]
struct Cell {
    block: bool
}

impl WindowContext {

}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        // let time = context.start.elapsed().as_secs_f32();
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);

        let (width, height) = canvas.dimensions();

        let default_program = canvas.shaders().borrow().default();
        let light_program = canvas.shaders().borrow().light();

        let uniforms = uniform! {
            mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport())
        };

        let s = S as f32;

        let color = [1.0; 4];

        let cursor = Light { pos: Vector2::from(context.mouse), color: context.color.clone(), radius: context.radius };

        for light in context.lights.iter().chain(once(&cursor)) {
            // Shadows
            for x in 0..GRID {
                for y in 0..GRID {
                    let cell = &context.grid[x][y];
                    if cell.block {
                        let x = x as f32 * s;
                        let y = y as f32 * s;
                        let vertices = [
                            vec2(x, y),
                            vec2(x + s, y),
                            vec2(x + s, y + s),
                            vec2(x, y + s)
                        ];
                        for i in 0..vertices.len() {
                            let this = vertices[i];
                            let next = vertices[(i + 1) % vertices.len()];
                            let edge = next - this;
                            let light_to_this = this - light.pos;
                            let light_to_next = next - light.pos;
                            if edge.perp_dot(light_to_this) > 0.0 {
                                let p1 = this + light_to_this * W as f32;
                                let p2 = next + light_to_next * W as f32;
                                canvas.generic_shape(&PrimitiveType::TriangleFan, &[
                                    Vertex::pos(this.extend(0.0)).color(color),
                                    Vertex::pos(p1.extend(0.0)).color(color),
                                    Vertex::pos(p2.extend(0.0)).color(color),
                                    Vertex::pos(next.extend(0.0)).color(color),
                                ], false, false, &default_program, &uniforms, &DrawParameters {
                                    color_mask: (false, false, false, false),
                                    stencil: Stencil {
                                        test_counter_clockwise: StencilTest::AlwaysPass,
                                        reference_value_counter_clockwise: 1,
                                        write_mask_counter_clockwise: 1,
                                        fail_operation_counter_clockwise: StencilOperation::Keep,
                                        pass_depth_fail_operation_counter_clockwise: StencilOperation::Keep,
                                        depth_pass_operation_counter_clockwise: StencilOperation::Keep,
                                        .. Default::default()
                                    },
                                    .. Default::default()
                                })
                            }
                        }

                        canvas.rect([x, y, s, s], color, &default_program, &uniforms, &Default::default());
                    }
                }
            }

            let viewport = canvas.viewport();

            let pos = light.pos;

            //Light source itself
            let uniforms = uniform! {
                mat: Into::<[[f32; 4]; 4]>::into(viewport),
                lightLocation: [pos.x, pos.y],
                lightColor: light.color,
                lightRadius: light.radius
            };
            let params = DrawParameters {
                blend: Blend::alpha_blending(),
                color_mask: (true, true, true, true),
                stencil: Stencil {
                    test_counter_clockwise: StencilTest::AlwaysPass,
                    reference_value_counter_clockwise: 0,
                    write_mask_counter_clockwise: 1,
                    fail_operation_counter_clockwise: StencilOperation::Keep,
                    pass_depth_fail_operation_counter_clockwise: StencilOperation::Keep,
                    depth_pass_operation_counter_clockwise: StencilOperation::Keep,
                    .. Default::default()
                },
                .. Default::default()
            };
            canvas.rect([0.0, 0.0, width, height], [1.0; 4], &light_program, &uniforms, &params);
            canvas.clear_stencil(0);
        }

        // Walls
        for x in 0..GRID {
            for y in 0..GRID {
                let cell = &context.grid[x][y];
                if cell.block {
                    let x = x as f32 * s;
                    let y = y as f32 * s;
                    canvas.rect([x, y, s, s], [0.0, 0.0, 0.0, 1.0], &default_program, &uniforms, &Default::default());
                }
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
                context.radius = (context.radius + y).clamp(2.0, 50.0);
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
        if button == MouseButton::Right && state == ElementState::Pressed {
            let mut cell = &mut context.grid[x][y];
            cell.block = !cell.block;
        }
        if button == MouseButton::Left && state == ElementState::Pressed {
            let color = context.color.clone();
            let radius = context.radius;
            context.lights.push(Light {
                pos: vec2(mx, my),
                color,
                radius
            });
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
            if key == VirtualKeyCode::Space && input.state == ElementState::Pressed {
                context.color = random();
            }
        }
    }
}

fn main() {
    window::create("Сапёр", LogicalSize::new(W, H), 24, WindowHandler);
}