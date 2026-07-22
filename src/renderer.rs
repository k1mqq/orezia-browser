use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, LayoutSettings, TextStyle};
use fontdue::{Font, Metrics};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use crate::html_parser::Dom;
use crate::layout::{Layout, LayoutContext};
use crate::styler::StyledTree;

struct Renderer {
    buffer: Vec<u32>,
    fonts: Vec<Font>,
    font_layout: fontdue::layout::Layout,
    font_cache: HashMap<GlyphRasterConfig, (Metrics, Vec<u8>)>,
    width: u32,
    height: u32,
}

struct App {
    renderer: Renderer,
    dom: Dom,
    state: Option<(Arc<Window>, softbuffer::Surface<Arc<Window>, Arc<Window>>)>,
}

impl Renderer {
    // it is unusable when initialized, because width and height are zero
    // draw is called with width and height, so it's ok
    fn new() -> Self {
        let font = include_bytes!("../resources/NotoSansCJKjp-Regular.otf") as &[u8];
        let settings = fontdue::FontSettings {
            scale: 200.0,
            ..fontdue::FontSettings::default()
        };

        let font = Font::from_bytes(font, settings).unwrap();

        let font_layout = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);
        Self {
            buffer: Vec::new(),
            fonts: vec![font],
            font_layout: font_layout,
            font_cache: HashMap::new(),
            width: 0,
            height: 0,
        }
    }

    fn draw(&mut self, buffer: &mut [u32], layout: Layout, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buffer = vec![u32::MAX; (width * height) as usize];

        self.draw_string("Orezia by k1mq!".to_string(), 100, 200, 1000, 100.0);
        self.draw_layout(layout);

        for (i, pixel) in buffer.iter_mut().enumerate() {
            let x = i % width as usize;
            let y = i / width as usize;

            *pixel = self.pixel_at(x, y);
        }
    }

    fn draw_layout(&mut self, layout: Layout) {
        let draw_calls: Vec<(String, usize, usize, usize)> = layout
            .components
            .iter()
            .filter(|c| {
                c.dimentions.content.x as u32 <= self.width
                    && c.dimentions.content.y as u32 <= self.height
            })
            .filter_map(|c| {
                if let Some(text) = &c.text {
                    Some((
                        text.clone(),
                        c.dimentions.content.x as usize,
                        c.dimentions.content.y as usize,
                        c.dimentions.content.width as usize,
                    ))
                } else {
                    None
                }
            })
            .collect();

        for (text, x, y, width) in draw_calls {
            self.draw_string(text, x, y, width, 20.0);
        }
    }

    // fn draw_char(&mut self, character: char, x: usize, y: usize, size: f32) {
    //     let (metrics, bitmap) = self.font.rasterize(character, size);

    //     for (i, bit) in bitmap.iter().enumerate() {
    //         let c = 255 - *bit;

    //         let ascent = metrics.height as i32 + metrics.bounds.ymin as i32;

    //         let x: usize = i % metrics.width + x;
    //         let y: usize = i / metrics.width + y - ascent as usize;

    //         self.buffer[y * self.width as usize + x] = rgb(c, c, c);
    //     }
    // }

    fn draw_string(&mut self, string: String, x: usize, y: usize, width: usize, size: f32) {
        let layout_settings = LayoutSettings {
            max_width: Some(width as f32),
            ..Default::default()
        };
        self.font_layout.reset(&layout_settings);
        // no longer cloned font
        self.font_layout
            .append(self.fonts.as_slice(), &TextStyle::new(&string, size, 0));
        for glyph in self.font_layout.glyphs() {
            let (metrics, bitmap) = self
                .font_cache
                .entry(glyph.key)
                .or_insert_with(|| self.fonts[0].rasterize_config(glyph.key));
            for (i, bit) in bitmap.iter().enumerate() {
                let c = 255 - *bit;

                let x: usize = i % metrics.width + x as usize + glyph.x as usize;
                let y: usize = i / metrics.width + y as usize + glyph.y as usize;

                if x >= self.width as usize || y >= self.height as usize {
                    break;
                }

                self.buffer[y * self.width as usize + x] = rgb(c, c, c);
            }
        }
    }

    fn pixel_at(&self, x: usize, y: usize) -> u32 {
        self.buffer[y * self.width as usize + x]
    }
}

impl App {
    fn new(renderer: Renderer, dom: Dom) -> Self {
        Self {
            renderer: renderer,
            dom: dom,
            state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let context = softbuffer::Context::new(Arc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Arc::clone(&window)).unwrap();

        self.state = Some((window, surface));
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(_) | WindowEvent::RedrawRequested => {
                let Some((window, surface)) = self.state.as_mut() else {
                    return;
                };

                let size = window.inner_size();

                let (w, h) = (size.width, size.height);
                if w == 0 || h == 0 {
                    return;
                }

                surface
                    .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
                    .unwrap();

                let mut buf = surface.buffer_mut().unwrap();

                let layout_context = LayoutContext {
                    font: &self.renderer.fonts[0],
                    window_height: h,
                    window_width: w,
                };

                let styled_tree = StyledTree::build(&self.dom);

                let layout = Layout::build(&styled_tree, layout_context);

                self.renderer.draw(&mut buf, layout, w, h);

                buf.present().unwrap();
            }
            _ => (),
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn render(dom: Dom) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let renderer = Renderer::new();
    let mut app = App::new(renderer, dom);

    event_loop.run_app(&mut app).unwrap();
}
