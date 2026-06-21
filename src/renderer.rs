use std::num::NonZeroU32;
use std::sync::Arc;

use fontdue::Font;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

struct App {
    font: Font,
    state: Option<(Arc<Window>, softbuffer::Surface<Arc<Window>, Arc<Window>>)>,
}

impl App {
    fn new() -> Self {
        let font = include_bytes!("../resources/NotoSansCJKjp-Regular.otf") as &[u8];
        let settings = fontdue::FontSettings {
            scale: 200.0,
            ..fontdue::FontSettings::default()
        };

        let font = Font::from_bytes(font, settings).unwrap();

        Self {
            font: font,
            state: None
        }
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

                draw(&mut buf, w);
                
                draw_char(&mut buf, w, &self.font, 'k');

                buf.present().unwrap();
            }
            _ => (),
        }
    }
}

fn draw(buffer:&mut [u32], width: u32) {
    buffer.fill(rgb(255, 255, 255));
}

fn draw_char(buffer: &mut [u32], width: u32, font: &Font, character: char) {
    let (metrics, bitmap) = font.rasterize(character, 200.0);

    for (i, bit) in bitmap.iter().enumerate() {
        let c = 255 - *bit;

        let x: usize = i % metrics.width;
        let y: usize = i / metrics.width;

        buffer[y * width as usize + x] = rgb(c, c, c);
    }
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn render() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}