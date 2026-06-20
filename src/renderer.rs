use std::num::NonZeroU32;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

#[derive(Default)]
struct App {
    state: Option<(Arc<Window>, softbuffer::Surface<Arc<Window>, Arc<Window>>)>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());

        let context = softbuffer::Context::new(Arc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Arc::clone(&window)).unwrap();

        self.state = Some((window, surface));
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    )
    {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::Resized(_) | WindowEvent::RedrawRequested => {
                let Some((window, surface)) = self.state.as_mut() else {
                    return;
                };

                let size = window.inner_size();

                let (w, h) = (size.width, size.height);
                if w == 0 || h == 0 {
                    return;
                }

                surface.resize(
                    NonZeroU32::new(w).unwrap(),
                    NonZeroU32::new(h).unwrap(),
                ).unwrap();

                let mut buf = surface.buffer_mut().unwrap();

                for (i, pixel) in buf.iter_mut().enumerate() {
                    let x = (i % w as usize) as u32;
                    let y = (i / w as usize) as u32;
                    *pixel = fill(x, y, w, h);
                }

                buf.present().unwrap();
            }
            _ => (),
        }
    }
}

fn fill(x: u32, y: u32, w: u32, h: u32) -> u32 {
    let r = (x * 255 / w) as u8;
    let g = (y * 255 / h) as u8;
    let b: u8 = 0x40;
    rgb(r, g, b)
}
 
#[inline]
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn render() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}