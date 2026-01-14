use crate::color::Color;

pub struct Canvas<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> Canvas<'a> {
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clear(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x as i32, y as i32, color);
            }
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = (y as u32 * self.width + x as u32) as usize * 4;
        self.buffer[offset] = color.b;
        self.buffer[offset + 1] = color.g;
        self.buffer[offset + 2] = color.r;
        self.buffer[offset + 3] = color.a;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color) {
        for py in y..(y + height) {
            for px in x..(x + width) {
                self.set_pixel(px, py, color);
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color, thickness: i32) {
        // Top
        self.fill_rect(x, y, width, thickness, color);
        // Bottom
        self.fill_rect(x, y + height - thickness, width, thickness, color);
        // Left
        self.fill_rect(x, y, thickness, height, color);
        // Right
        self.fill_rect(x + width - thickness, y, thickness, height, color);
    }

    pub fn draw_shadow(&mut self, x: i32, y: i32, width: i32, height: i32, blur: i32, color: Color) {
        // Simple shadow effect using multiple layers with decreasing opacity
        for i in 0..blur {
            let offset = blur - i;
            let alpha = (color.a as f32 * (i as f32 / blur as f32) * 0.3) as u8;
            let shadow_color = Color::rgba(color.r, color.g, color.b, alpha);
            
            // Draw shadow layers
            self.draw_rect(
                x + offset,
                y + offset,
                width,
                height,
                shadow_color,
                1,
            );
        }
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = (y as u32 * self.width + x as u32) as usize * 4;
        let alpha = color.a as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        self.buffer[offset] = ((self.buffer[offset] as f32 * inv_alpha) + (color.b as f32 * alpha)) as u8;
        self.buffer[offset + 1] = ((self.buffer[offset + 1] as f32 * inv_alpha) + (color.g as f32 * alpha)) as u8;
        self.buffer[offset + 2] = ((self.buffer[offset + 2] as f32 * inv_alpha) + (color.r as f32 * alpha)) as u8;
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.buffer
    }
}
