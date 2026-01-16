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
            renderer_name: "LLVMpipe Software Renderer".to_string(),
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
        // Limit blur radius for performance (max 8 pixels)
        let blur = blur.min(8);
        
        // Skip if blur is too small to be visible
        if blur < 2 {
            return;
        }
        
        // Step 1: Create alpha-only texture for the shape
        let mut alpha_map = vec![0u8; (width * height) as usize];
        
        // Fill the shape area with full alpha
        for idx in 0..(width * height) as usize {
            alpha_map[idx] = 255;
        }
        
        // Step 2: Blur the alpha texture (fast box blur with reduced quality)
        let mut blurred_alpha = alpha_map.clone();
        
        // Use smaller blur radius for performance (divide by 2)
        let fast_blur = (blur / 2).max(1);
        
        // Horizontal pass
        let mut temp = vec![0u8; (width * height) as usize];
        for py in 0..height {
            for px in 0..width {
                let mut sum = 0u32;
                let mut count = 0u32;
                
                let start_x = (px - fast_blur).max(0);
                let end_x = (px + fast_blur).min(width - 1);
                
                for sample_x in start_x..=end_x {
                    let idx = (py * width + sample_x) as usize;
                    sum += alpha_map[idx] as u32;
                    count += 1;
                }
                
                let idx = (py * width + px) as usize;
                temp[idx] = (sum / count) as u8;
            }
        }
        
        // Vertical pass
        for py in 0..height {
            for px in 0..width {
                let mut sum = 0u32;
                let mut count = 0u32;
                
                let start_y = (py - fast_blur).max(0);
                let end_y = (py + fast_blur).min(height - 1);
                
                for sample_y in start_y..=end_y {
                    let idx = (sample_y * width + px) as usize;
                    sum += temp[idx] as u32;
                    count += 1;
                }
                
                let idx = (py * width + px) as usize;
                blurred_alpha[idx] = (sum / count) as u8;
            }
        }
        
        // Step 3: Multiply shadow color by blurred alpha and composite
        let shadow_offset = blur / 2;
        for py in 0..height {
            for px in 0..width {
                let idx = (py * width + px) as usize;
                let alpha = blurred_alpha[idx];
                
                // Skip fully transparent pixels
                if alpha < 5 {
                    continue;
                }
                
                // Apply shadow color with blurred alpha
                let shadow_alpha = ((alpha as f32 / 255.0) * (color.a as f32 / 255.0) * 255.0) as u8;
                let shadow_color = Color::rgba(color.r, color.g, color.b, shadow_alpha);
                
                // Composite under the card (offset by shadow distance)
                self.blend_pixel(
                    x + px + shadow_offset,
                    y + py + shadow_offset,
                    shadow_color,
                );
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
        let shadow_start = std::time::Instant::now();
        
        // Limit blur radius for performance (max 8 pixels)
        let blur = blur.min(8);
        
        // Skip if blur is too small to be visible
        if blur < 2 {
            return;
        }
        
        debug_log!("draw_rounded_shadow: {}x{} at ({},{}) blur={} radius={}", 
                  width, height, x, y, blur, radius);
        
        // Step 1: Create alpha-only texture for rounded rect shape
        let mut alpha_map = vec![0u8; (width * height) as usize];
        let radius = radius as i32;
        
        // Fill main body
        for py in 0..height {
            for px in 0..width {
                let idx = (py * width + px) as usize;
                
                // Check if pixel is inside rounded rectangle
                let in_main_body = (px >= radius && px < width - radius) ||
                                   (py >= radius && py < height - radius);
                
                if in_main_body {
                    alpha_map[idx] = 255;
                } else {
                    // Check corners
                    let corners = [
                        (radius, radius),                    // Top-left
                        (width - radius, radius),            // Top-right
                        (radius, height - radius),           // Bottom-left
                        (width - radius, height - radius),   // Bottom-right
                    ];
                    
                    for (cx, cy) in corners {
                        let dx = px - cx;
                        let dy = py - cy;
                        let dist_sq = dx * dx + dy * dy;
                        let radius_sq = radius * radius;
                        
                        if dist_sq <= radius_sq {
                            alpha_map[idx] = 255;
                            break;
                        }
                    }
                }
            }
        }
        
        // Step 2: Blur the alpha texture (fast box blur with reduced quality)
        let mut blurred_alpha = alpha_map.clone();
        
        // Use smaller blur radius for performance (divide by 2)
        let fast_blur = (blur / 2).max(1);
        
        // Horizontal pass
        let mut temp = vec![0u8; (width * height) as usize];
        for py in 0..height {
            for px in 0..width {
                let mut sum = 0u32;
                let mut count = 0u32;
                
                let start_x = (px - fast_blur).max(0);
                let end_x = (px + fast_blur).min(width - 1);
                
                for sample_x in start_x..=end_x {
                    let idx = (py * width + sample_x) as usize;
                    sum += alpha_map[idx] as u32;
                    count += 1;
                }
                
                let idx = (py * width + px) as usize;
                temp[idx] = (sum / count) as u8;
            }
        }
        
        // Vertical pass
        for py in 0..height {
            for px in 0..width {
                let mut sum = 0u32;
                let mut count = 0u32;
                
                let start_y = (py - fast_blur).max(0);
                let end_y = (py + fast_blur).min(height - 1);
                
                for sample_y in start_y..=end_y {
                    let idx = (sample_y * width + px) as usize;
                    sum += temp[idx] as u32;
                    count += 1;
                }
                
                let idx = (py * width + px) as usize;
                blurred_alpha[idx] = (sum / count) as u8;
            }
        }
        
        // Step 3: Multiply shadow color by blurred alpha and composite
        let shadow_offset = blur / 2;
        for py in 0..height {
            for px in 0..width {
                let idx = (py * width + px) as usize;
                let alpha = blurred_alpha[idx];
                
                // Skip fully transparent pixels
                if alpha < 5 {
                    continue;
                }
                
                // Apply shadow color with blurred alpha
                let shadow_alpha = ((alpha as f32 / 255.0) * (color.a as f32 / 255.0) * 255.0) as u8;
                let shadow_color = Color::rgba(color.r, color.g, color.b, shadow_alpha);
                
                // Composite under the card (offset by shadow distance)
                self.blend_pixel(
                    x + px + shadow_offset,
                    y + py + shadow_offset,
                    shadow_color,
                );
            }
        }
        
        let shadow_elapsed = shadow_start.elapsed();
        debug_log!("Shadow rendering took: {:.2}ms", shadow_elapsed.as_secs_f64() * 1000.0);
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

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.buffer
    }
}
