use std::num::NonZeroU32;
use std::sync::Arc;

use fontdue::Font;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

struct Renderer {
    font: Font,
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

struct App {
    renderer: Option<Renderer>,
    state: Option<(Arc<Window>, softbuffer::Surface<Arc<Window>, Arc<Window>>)>,
}

impl Renderer {
    fn new(width: u32, height: u32) -> Self {
        let font = include_bytes!("../resources/NotoSansCJKjp-Regular.otf") as &[u8];
        let settings = fontdue::FontSettings {
            scale: 200.0,
            ..fontdue::FontSettings::default()
        };

        let font = Font::from_bytes(font, settings).unwrap();

        Self {
            font: font,
            buffer: vec![u32::MAX; (width * height) as usize],
            width: width,
            height: height,
        }
    }

    fn draw(&mut self, buffer: &mut [u32], width: u32, height: u32){
        self.width = width;
        self.height = height;
        self.buffer = vec![u32::MAX; (width * height) as usize];

        self.draw_char('k', 100, 100, 100.0);
        self.draw_string("Orezia by k1mq!".to_string(), 100, 200, 100.0);

        for (i, pixel) in buffer.iter_mut().enumerate() {
            let x = i % width as usize;
            let y = i / width as usize;

            *pixel = self.pixel_at(x, y);
        }
    }

    fn draw_char(&mut self, character: char, x: usize, y: usize, size: f32) {
        let (metrics, bitmap) = self.font.rasterize(character, size);

        for (i, bit) in bitmap.iter().enumerate() {
            let c = 255 - *bit;

            let ascent = metrics.height as i32 + metrics.bounds.ymin as i32;

            let x: usize = i % metrics.width + x;
            let y: usize = i / metrics.width + y - ascent as usize;

            self.buffer[y * self.width as usize + x] = rgb(c, c, c);
        }
    }

    fn draw_string(&mut self, string: String, x: usize, y: usize, size: f32) {
        let mut x = x;
        // let mut y = y;
        for ch in string.chars() {
            let metrics = self.font.metrics(ch, size);
            self.draw_char(ch, x, y, size);

            x = x + metrics.advance_width as usize;
        }
    }

    fn pixel_at(&self, x: usize, y: usize) -> u32{
        self.buffer[y * self.width as usize + x]
    }
}

impl App {
    fn new() -> Self {
        Self {
            renderer: None,
            state: None
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());

        let context = softbuffer::Context::new(Arc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Arc::clone(&window)).unwrap();

        self.renderer = Some(Renderer::new(window.inner_size().width, window.inner_size().height));
        self.state = Some((window, surface));
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
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

                let Some(renderer)  = self.renderer.as_mut() else {
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

                renderer.draw(&mut buf, w, h);
                
                buf.present().unwrap();
            }
            _ => (),
        }
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