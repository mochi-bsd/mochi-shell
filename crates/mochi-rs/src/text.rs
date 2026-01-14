use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use crate::{canvas::Canvas, color::Color};

pub struct TextRenderer {
    fonts: HashMap<String, Font>,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    pub fn load_font(&mut self, name: &str, font_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let font = Font::from_bytes(font_data, FontSettings::default())?;
        self.fonts.insert(name.to_string(), font);
        Ok(())
    }

    pub fn render(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: i32,
        y: i32,
        font_size: f32,
        color: Color,
        font_name: &str,
    ) {
        let font = match self.fonts.get(font_name) {
            Some(f) => f,
            None => {
                eprintln!("Font '{}' not found", font_name);
                return;
            }
        };

        let mut cursor_x = x;
        let baseline_offset = self.calculate_baseline(text, font_size, font);
        
        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, font_size);
            let advance = metrics.advance_width as i32;
            
            if metrics.width > 0 && metrics.height > 0 {
                let char_x = cursor_x + metrics.xmin;
                let char_y = y + baseline_offset - metrics.height as i32 - metrics.ymin;

                for py in 0..metrics.height {
                    for px in 0..metrics.width {
                        let screen_x = char_x + px as i32;
                        let screen_y = char_y + py as i32;

                        let bitmap_idx = py * metrics.width + px;
                        let alpha = bitmap[bitmap_idx];
                        
                        if alpha > 0 {
                            let pixel_color = Color::rgba(color.r, color.g, color.b, alpha);
                            canvas.blend_pixel(screen_x, screen_y, pixel_color);
                        }
                    }
                }
            }
            
            cursor_x += advance;
        }
    }

    fn calculate_baseline(&self, text: &str, font_size: f32, font: &Font) -> i32 {
        let mut max_ascent = 0;
        
        for ch in text.chars() {
            let (metrics, _) = font.rasterize(ch, font_size);
            let ascent = metrics.height as i32 + metrics.ymin;
            max_ascent = max_ascent.max(ascent);
        }
        
        max_ascent
    }

    pub fn measure(&self, text: &str, font_size: f32, font_name: &str) -> (i32, i32) {
        let font = match self.fonts.get(font_name) {
            Some(f) => f,
            None => return (0, 0),
        };

        let mut width = 0;
        let mut max_height = 0;
        
        for ch in text.chars() {
            let (metrics, _) = font.rasterize(ch, font_size);
            width += metrics.advance_width as i32;
            max_height = max_height.max(metrics.height as i32);
        }
        
        (width, max_height)
    }
}
