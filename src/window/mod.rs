use std::time::{Duration, Instant};
use glium::{Display, Frame};
use glium::glutin::dpi::Size;
use glium::glutin::event::{Event, ModifiersState, WindowEvent};
use crate::{ContextBuilder, ControlFlow, EventLoop, KeyboardInput, MouseScrollDelta, StartCause, WindowBuilder};

pub fn create<T, S, C, H>(title: T, inner_size: S, depth_bits: u8, mut handler: H)
    where T: Into<String>,
          S: Into<Size>,
          C: Context + 'static,
          H: Handler<C> + 'static
{
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(inner_size);

    let cb = ContextBuilder::new().with_depth_buffer(depth_bits);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let mut context = C::new(&display);

    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        let fps_limit = context.get_frame_limit();
        let next_frame = Instant::now() + Duration::from_secs_f32(1.0 / fps_limit);
        *control_flow = ControlFlow::WaitUntil(next_frame);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    handler.on_keyboard_input(&mut context, input);
                },
                WindowEvent::MouseWheel { delta, modifiers, .. } => {
                    handler.on_mouse_scroll(&mut context, delta, modifiers);
                },
                WindowEvent::Resized(size) => {
                    let dpi = display.gl_window().window().scale_factor();
                    let size = size.to_logical::<f32>(dpi);
                    handler.on_resized(&mut context, size.width, size.height);
                }
                _ => {
                    *control_flow = ControlFlow::Poll;
                },
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => {},
                StartCause::Init => {},
                StartCause::WaitCancelled { .. } => {},
                _ => return,
            }
            _ => return,
        };

        let now = Instant::now();
        let time_elapsed = now.duration_since(start_time).as_secs_f32();

        let mut frame = display.draw();

        handler.draw_frame(&mut context, &mut frame, time_elapsed);

        frame.finish().unwrap();
    });
}

pub trait Context: Sized {
    fn new(display: &Display) -> Self;

    fn get_frame_limit(&self) -> f32 {
        60.0
    }
}

pub trait Handler<C: Context>: Sized {
    fn draw_frame(&mut self, context: &mut C, frame: &mut Frame, time_elapsed: f32);

    fn on_keyboard_input(&mut self, context: &mut C, input: KeyboardInput) {

    }

    fn on_mouse_scroll(&mut self, context: &mut C, delta: MouseScrollDelta, modifiers: ModifiersState) {

    }

    fn on_resized(&mut self, context: &mut C, width: f32, height: f32) {

    }
}