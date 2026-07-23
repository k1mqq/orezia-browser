use std::collections::HashMap;

use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, LayoutSettings, TextStyle};
use fontdue::{Font, Metrics};

use crate::layout::Layout;
use crate::layout::Text;

struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }
}

impl From<u32> for Color {
    fn from(n: u32) -> Self {
        Color {
            r: (n >> 16 & 0xff) as u8,
            g: (n >> 8 & 0xff) as u8,
            b: (n & 0xff) as u8,
        }
    }
}

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

        self.draw_string(
            "Orezia by k1mq!".to_string(),
            Rect {
                x: 100,
                y: 200,
                width: 1000,
                height: 100,
            },
            100.0,
            Color {
                r: 0,
                g: 200,
                b: 200,
            },
        );
        self.draw_layout(layout);

        for (i, pixel) in buffer.iter_mut().enumerate() {
            let x = i % width as usize;
            let y = i / width as usize;

            *pixel = self.pixel_at(x, y);
        }
    }

    fn draw_layout(&mut self, layout: Layout) {
        let draw_calls: Vec<(Text, Rect)> = layout
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
                        Rect {
                            x: c.dimentions.content.x as usize,
                            y: c.dimentions.content.y as usize,
                            width: c.dimentions.content.width as usize,
                            height: c.dimentions.content.height as usize,
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();

        for (text, text_box) in draw_calls {
            self.draw_string(text.text, text_box, text.size, text.color);
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

    fn draw_string(&mut self, text: String, text_box: Rect, size: f32, color: Color) {
        let layout_settings = LayoutSettings {
            max_width: Some(text_box.width as f32),
            ..Default::default()
        };
        self.font_layout.reset(&layout_settings);
        // no longer cloned font
        self.font_layout
            .append(self.fonts.as_slice(), &TextStyle::new(&text, size, 0));
        for glyph in self.font_layout.glyphs() {
            let (metrics, bitmap) = self
                .font_cache
                .entry(glyph.key)
                .or_insert_with(|| self.fonts[0].rasterize_config(glyph.key));
            for (i, bit) in bitmap.iter().enumerate() {
                if *bit == 0 {
                    continue;
                }

                let x: usize = i % metrics.width + text_box.x as usize + glyph.x as usize;
                let y: usize = i / metrics.width + text_box.y as usize + glyph.y as usize;

                if x >= self.width as usize || y >= self.height as usize {
                    continue;
                }

                let index = y * self.width as usize + x;

                self.buffer[index] = blend(color, Color::from(self.buffer[index]), *bit).into();
            }
        }
    }

    fn pixel_at(&self, x: usize, y: usize) -> u32 {
        self.buffer[y * self.width as usize + x]
    }
}

fn blend(fg: Color, bg: Color, a: u8) -> Color {
    let a = a as u16;
    let inv_a = 255 - a;

    let r = (fg.r as u16 * a + bg.r as u16 * inv_a) / 255;
    let g = (fg.g as u16 * a + bg.g as u16 * inv_a) / 255;
    let b = (fg.b as u16 * a + bg.b as u16 * inv_a) / 255;

    Color {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    }
}
