use crate::core::color::Color;

// Debug logging macro
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("MOCHI_DEBUG").is_ok() {
            eprintln!("[CANVAS] {}", format!($($arg)*));
        }
    };
}

use glam::Vec2;

pub struct Canvas<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
    renderer_name: String,
}

impl<'a> Canvas<'a> {
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
            renderer_name: "LLVMpipe (Mesa Software Renderer)".to_string(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_gpu_info(&mut self, _backend: String, device: String) {
        self.renderer_name = device;
    }

    pub fn get_renderer_type(&self) -> &str {
        "Software"
    }

    pub fn get_device_name(&self) -> &str {
        &self.renderer_name
    }

    pub fn clear(&mut self, color: Color) {
        for chunk in self.buffer.chunks_exact_mut(4) {
            chunk[0] = color.b;
            chunk[1] = color.g;
            chunk[2] = color.r;
            chunk[3] = color.a;
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

    pub fn fill_rounded_rect(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        radius: f32,
        color: Color,
    ) {
        let radius = radius as i32;

        // Fill main body
        self.fill_rect(x + radius, y, width - radius * 2, height, color);
        self.fill_rect(x, y + radius, width, height - radius * 2, color);

        // Draw corners with anti-aliasing
        for corner_x in [x, x + width - radius] {
            for corner_y in [y, y + height - radius] {
                for dy in 0..radius {
                    for dx in 0..radius {
                        let center_x = if corner_x == x { radius } else { 0 };
                        let center_y = if corner_y == y { radius } else { 0 };

                        let dist = (((dx - center_x) * (dx - center_x)
                            + (dy - center_y) * (dy - center_y))
                            as f32)
                            .sqrt();

                        if dist <= radius as f32 {
                            let alpha = if dist >= radius as f32 - 1.0 {
                                ((radius as f32 - dist) * 255.0) as u8
                            } else {
                                255
                            };

                            let aa_color = Color::rgba(color.r, color.g, color.b, 
                                ((alpha as f32 / 255.0) * (color.a as f32 / 255.0) * 255.0) as u8);
                            self.blend_pixel(corner_x + dx, corner_y + dy, aa_color);
                        }
                    }
                }
            }
        }
    }

    pub fn fill_gradient_rect(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_color: Color,
        end_color: Color,
        angle: f32,
    ) {
        let angle_rad = angle.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        for py in 0..height {
            for px in 0..width {
                let fx = px as f32 / width as f32;
                let fy = py as f32 / height as f32;

                let t = (fx * cos_a + fy * sin_a).clamp(0.0, 1.0);

                let r = (start_color.r as f32 * (1.0 - t) + end_color.r as f32 * t) as u8;
                let g = (start_color.g as f32 * (1.0 - t) + end_color.g as f32 * t) as u8;
                let b = (start_color.b as f32 * (1.0 - t) + end_color.b as f32 * t) as u8;
                let a = (start_color.a as f32 * (1.0 - t) + end_color.a as f32 * t) as u8;

                self.set_pixel(x + px, y + py, Color::rgba(r, g, b, a));
            }
        }
    }

    pub fn draw_rect(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
        thickness: i32,
    ) {
        // Top
        self.fill_rect(x, y, width, thickness, color);
        // Bottom
        self.fill_rect(x, y + height - thickness, width, thickness, color);
        // Left
        self.fill_rect(x, y, thickness, height, color);
        // Right
        self.fill_rect(x + width - thickness, y, thickness, height, color);
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            self.set_pixel(x, y, color);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_shadow(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        blur: i32,
        color: Color,
    ) {
        // Limit blur radius for performance on software renderer
        let blur = blur.min(6).max(0);
        
        if blur < 1 {
            return;
        }
        
        // Small offset for shadow
        let shadow_offset = (blur / 2).max(1);
        let blur_f = blur as f32;
        
        // Render shadow with simple linear falloff - only outside the shape
        for py in -blur..=(height + blur) {
            for px in -blur..=(width + blur) {
                // Calculate distance to nearest edge
                let dx = if px < 0 {
                    -px as f32
                } else if px >= width {
                    (px - width + 1) as f32
                } else {
                    0.0
                };
                
                let dy = if py < 0 {
                    -py as f32
                } else if py >= height {
                    (py - height + 1) as f32
                } else {
                    0.0
                };
                
                let dist = (dx * dx + dy * dy).sqrt();
                
                // Only render shadow OUTSIDE the shape
                if dist <= 0.0 || dist > blur_f {
                    continue;
                }
                
                // Simple linear falloff (no smoothstep, no pow)
                let t = (dist / blur_f).min(1.0);
                let falloff = 1.0 - t;
                
                // Clamp alpha to reasonable range
                let shadow_alpha = (falloff * 0.4 * 255.0).min(100.0) as u8;
                
                if shadow_alpha > 1 {
                    let shadow_color = Color::rgba(color.r, color.g, color.b, shadow_alpha);
                    self.blend_pixel_premul(
                        x + px + shadow_offset,
                        y + py + shadow_offset,
                        shadow_color,
                    );
                }
            }
        }
    }

    pub fn draw_rounded_shadow(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        radius: f32,
        blur: i32,
        color: Color,
    ) {
        // Limit blur radius for performance on software renderer
        let blur = blur.min(6).max(0);
        
        if blur < 1 {
            return;
        }
        
        debug_log!("draw_rounded_shadow: {}x{} at ({},{}) blur={} radius={}", 
                  width, height, x, y, blur, radius);
        
        let shadow_start = std::time::Instant::now();
        
        // Small offset and radius for better performance
        let shadow_offset = (blur / 2).max(1);
        let blur_f = blur as f32;
        let radius_i = (radius.min(8.0)) as i32;
        
        // Render shadow with simple linear falloff - only outside the shape
        for py in -blur..=(height + blur) {
            for px in -blur..=(width + blur) {
                // Calculate distance to nearest point on the rounded rectangle
                let dist = self.distance_to_rounded_rect(px, py, width, height, radius_i);
                
                // Only render shadow OUTSIDE the shape (dist > 0)
                if dist <= 0.0 || dist > blur_f {
                    continue;
                }
                
                // Simple linear falloff (no smoothstep, no pow)
                let t = (dist / blur_f).min(1.0);
                let falloff = 1.0 - t;
                
                // Clamp alpha to reasonable range
                let shadow_alpha = (falloff * 0.4 * 255.0).min(100.0) as u8;
                
                if shadow_alpha > 1 {
                    let shadow_color = Color::rgba(color.r, color.g, color.b, shadow_alpha);
                    self.blend_pixel_premul(
                        x + px + shadow_offset,
                        y + py + shadow_offset,
                        shadow_color,
                    );
                }
            }
        }
        
        let shadow_elapsed = shadow_start.elapsed();
        debug_log!("Shadow rendering took: {:.2}ms", shadow_elapsed.as_secs_f64() * 1000.0);
    }
    
    // Helper function to calculate signed distance from a point to a rounded rectangle
    // Returns negative if inside, positive if outside
    fn distance_to_rounded_rect(&self, px: i32, py: i32, width: i32, height: i32, radius: i32) -> f32 {
        // Check if we're in a corner region
        let in_top_left = px < radius && py < radius;
        let in_top_right = px >= width - radius && py < radius;
        let in_bottom_left = px < radius && py >= height - radius;
        let in_bottom_right = px >= width - radius && py >= height - radius;
        
        if in_top_left {
            let dx = px - radius;
            let dy = py - radius;
            let corner_dist = ((dx * dx + dy * dy) as f32).sqrt();
            corner_dist - radius as f32
        } else if in_top_right {
            let dx = px - (width - radius - 1);
            let dy = py - radius;
            let corner_dist = ((dx * dx + dy * dy) as f32).sqrt();
            corner_dist - radius as f32
        } else if in_bottom_left {
            let dx = px - radius;
            let dy = py - (height - radius - 1);
            let corner_dist = ((dx * dx + dy * dy) as f32).sqrt();
            corner_dist - radius as f32
        } else if in_bottom_right {
            let dx = px - (width - radius - 1);
            let dy = py - (height - radius - 1);
            let corner_dist = ((dx * dx + dy * dy) as f32).sqrt();
            corner_dist - radius as f32
        } else {
            // Not in a corner, calculate distance to nearest edge
            let dx = if px < 0 {
                -px as f32
            } else if px >= width {
                (px - width + 1) as f32
            } else {
                0.0
            };
            
            let dy = if py < 0 {
                -py as f32
            } else if py >= height {
                (py - height + 1) as f32
            } else {
                0.0
            };
            
            // If inside the main body, return negative distance
            if dx == 0.0 && dy == 0.0 {
                -1.0
            } else {
                (dx * dx + dy * dy).sqrt()
            }
        }
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = (y as u32 * self.width + x as u32) as usize * 4;
        let alpha = color.a as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        self.buffer[offset] =
            ((self.buffer[offset] as f32 * inv_alpha) + (color.b as f32 * alpha)) as u8;
        self.buffer[offset + 1] =
            ((self.buffer[offset + 1] as f32 * inv_alpha) + (color.g as f32 * alpha)) as u8;
        self.buffer[offset + 2] =
            ((self.buffer[offset + 2] as f32 * inv_alpha) + (color.r as f32 * alpha)) as u8;
    }
    
    // Premultiplied alpha blending - optimized for software rendering
    pub fn blend_pixel_premul(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = (y as u32 * self.width + x as u32) as usize * 4;
        
        // Clamp alpha
        let alpha = (color.a as u32).min(255);
        let inv_alpha = 255 - alpha;
        
        // Premultiplied alpha blend using integer math (faster on software renderer)
        self.buffer[offset] = 
            (((self.buffer[offset] as u32 * inv_alpha) + (color.b as u32 * alpha)) / 255) as u8;
        self.buffer[offset + 1] = 
            (((self.buffer[offset + 1] as u32 * inv_alpha) + (color.g as u32 * alpha)) / 255) as u8;
        self.buffer[offset + 2] = 
            (((self.buffer[offset + 2] as u32 * inv_alpha) + (color.r as u32 * alpha)) / 255) as u8;
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.buffer
    }
}
