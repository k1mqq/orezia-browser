use std::collections::HashMap;

use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, LayoutSettings, TextStyle};
use fontdue::{Font, Metrics};

use crate::layout::Layout;

pub struct Renderer {
    buffer: Vec<u32>,
    pub fonts: Vec<Font>,
    font_layout: fontdue::layout::Layout,
    font_cache: HashMap<GlyphRasterConfig, (Metrics, Vec<u8>)>,
    width: u32,
    height: u32,
}

impl Renderer {
    // it is unusable when initialized, because width and height are zero
    // draw is called with width and height, so it's ok
    pub fn new() -> Self {
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

    pub fn draw(&mut self, buffer: &mut [u32], layout: Layout, width: u32, height: u32) {
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

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
