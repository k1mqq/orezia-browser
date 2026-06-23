use std::num::NonZeroU32;
use std::sync::Arc;

use fontdue::{Font, Metrics};
use fontdue::layout::{CoordinateSystem, LayoutSettings, TextStyle};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use crate::layout::{Content, Layout};

struct Renderer {
    font: Font,
    buffer: Vec<u32>,
    layout: Layout,
    font_layout: fontdue::layout::Layout,
    width: u32,
    height: u32,
}

struct App {
    renderer: Renderer,
    state: Option<(Arc<Window>, softbuffer::Surface<Arc<Window>, Arc<Window>>)>,
}

impl Renderer {
    // it is unusable when initialized, because width and height are zero
    // draw is called with width and height, so it's ok
    fn new(layout: Layout) -> Self {
        let font = include_bytes!("../resources/NotoSansCJKjp-Regular.otf") as &[u8];
        let settings = fontdue::FontSettings {
            scale: 200.0,
            ..fontdue::FontSettings::default()
        };

        let font = Font::from_bytes(font, settings).unwrap();

        let font_layout = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);

        Self {
            font: font,
            buffer: Vec::new(),
            layout: layout,
            font_layout: font_layout,
            width: 0,
            height: 0,
        }
    }

    fn draw(&mut self, buffer: &mut [u32], width: u32, height: u32){
        self.width = width;
        self.height = height;
        self.buffer = vec![u32::MAX; (width * height) as usize];

        self.draw_char('k', 100, 100, 100.0);
        self.draw_string("Orezia by k1mq!".to_string(), 100, 200, 100.0);
        self.draw_layout();

        for (i, pixel) in buffer.iter_mut().enumerate() {
            let x = i % width as usize;
            let y = i / width as usize;

            *pixel = self.pixel_at(x, y);
        }
    }

    fn draw_layout(&mut self) {
        let draw_calls: Vec<(String, usize, usize)> = self.layout.components
            .iter()
            .filter_map(|c| {
                if let Some(Content::Text(ref text)) = c.content {
                    Some((text.clone(), c.rect.x as usize, c.rect.y as usize))
                } else {
                    None
                }
            }).collect();
        
        for (text, x, y) in draw_calls {
            self.draw_string(text, x, y, 20.0);
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
        self.font_layout.reset(&LayoutSettings::default());
        // is this clone ok?
        self.font_layout.append(&[self.font.clone()], &TextStyle::new(&string, size, 0));
        for glyph in self.font_layout.glyphs() {
            let (metrics, bitmap) = self.font.rasterize_config(glyph.key);
            for (i, bit) in bitmap.iter().enumerate() {
                let c = 255 - *bit;

                let x: usize = i % metrics.width + x as usize + glyph.x as usize;
                let y: usize = i / metrics.width + y as usize + glyph.y as usize;

                self.buffer[y * self.width as usize + x] = rgb(c, c, c);
            }
        }
    }

    fn pixel_at(&self, x: usize, y: usize) -> u32{
        self.buffer[y * self.width as usize + x]
    }
}

impl App {
    fn new(renderer: Renderer) -> Self {
        Self {
            renderer: renderer,
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

                self.renderer.draw(&mut buf, w, h);
                
                buf.present().unwrap();
            }
            _ => (),
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn render(layout: Layout) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut renderer = Renderer::new(layout);
    let mut app = App::new(renderer);

    event_loop.run_app(&mut app).unwrap();
}