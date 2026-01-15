use crate::core::color::Color;
use crate::core::gpu::{ShaderEffect, FragmentShader, ShaderContext, GpuBackend, GpuBackendType};
use glam::Vec2;

pub enum RenderBackend {
    CPU,
    GPU(Box<dyn GpuBackend>),
}

pub struct Canvas<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
    backend: Option<RenderBackend>,
    gpu_backend_name: String,
    gpu_device_name: String,
}

impl<'a> Canvas<'a> {
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
            backend: None,
            gpu_backend_name: "CPU".to_string(),
            gpu_device_name: "Software Renderer".to_string(),
        }
    }

    /// Set GPU info for display purposes
    pub fn set_gpu_info(&mut self, backend_name: String, device_name: String) {
        self.gpu_backend_name = backend_name;
        self.gpu_device_name = device_name;
    }

    /// Try to initialize GPU backend (Vulkan > OpenGL > GLES)
    pub fn try_init_gpu(&mut self) -> bool {
        // TODO: Implement GPU backend initialization
        // For now, we'll use CPU rendering
        // In the future:
        // 1. Try Vulkan first
        // 2. Fall back to OpenGL
        // 3. Fall back to OpenGL ES
        // 4. Fall back to CPU
        false
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn has_gpu(&self) -> bool {
        matches!(self.backend, Some(RenderBackend::GPU(_)))
    }

    /// Get the current renderer backend type
    pub fn get_renderer_type(&self) -> String {
        self.gpu_backend_name.clone()
    }

    /// Get GPU device name (if available)
    pub fn get_device_name(&self) -> String {
        self.gpu_device_name.clone()
    }

    /// Execute a custom GLSL-like fragment shader (CPU or GPU)
    pub fn execute_shader(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        shader: &dyn FragmentShader,
    ) {
        if self.has_gpu() {
            // GPU path: compile and execute shader on GPU
            // TODO: Implement GPU shader execution
            self.execute_shader_cpu(x, y, width, height, shader);
        } else {
            // CPU path: execute shader per-pixel
            self.execute_shader_cpu(x, y, width, height, shader);
        }
    }

    /// Execute shader on CPU (software rendering)
    fn execute_shader_cpu(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        shader: &dyn FragmentShader,
    ) {
        let mut ctx = ShaderContext::new(Vec2::new(width as f32, height as f32));
        
        for py in 0..height {
            for px in 0..width {
                ctx.frag_coord = Vec2::new(px as f32, py as f32);
                let color = shader.fragment(&ctx);
                
                // Convert Vec4 (0.0-1.0) to Color (0-255)
                let r = (color.x * 255.0).clamp(0.0, 255.0) as u8;
                let g = (color.y * 255.0).clamp(0.0, 255.0) as u8;
                let b = (color.z * 255.0).clamp(0.0, 255.0) as u8;
                let a = (color.w * 255.0).clamp(0.0, 255.0) as u8;
                
                if a > 0 {
                    let pixel_color = Color::rgba(r, g, b, a);
                    if a == 255 {
                        self.set_pixel(x + px, y + py, pixel_color);
                    } else {
                        self.blend_pixel(x + px, y + py, pixel_color);
                    }
                }
            }
        }
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

    pub fn fill_rect_with_effect(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
        effect: &ShaderEffect,
    ) {
        match effect.effect_type {
            crate::core::gpu::shader::ShaderEffectType::Blur => {
                // Software blur: draw with slight transparency and offset
                self.fill_rect(x, y, width, height, color);
                let blur_color = Color::rgba(color.r, color.g, color.b, 30);
                for offset in 1..3 {
                    self.fill_rect(x - offset, y - offset, width + offset * 2, height + offset * 2, blur_color);
                }
            }
            crate::core::gpu::shader::ShaderEffectType::Glow => {
                // Software glow: draw base + brighter outer glow
                let glow_intensity = effect.parameters.glow_intensity;
                let glow_r = (color.r as f32 * glow_intensity).min(255.0) as u8;
                let glow_g = (color.g as f32 * glow_intensity).min(255.0) as u8;
                let glow_b = (color.b as f32 * glow_intensity).min(255.0) as u8;
                
                // Draw glow layers
                for i in (0..6).rev() {
                    let alpha = (50.0 * (i as f32 / 6.0)) as u8;
                    let glow_color = Color::rgba(glow_r, glow_g, glow_b, alpha);
                    self.fill_rect(x - i, y - i, width + i * 2, height + i * 2, glow_color);
                }
                
                // Draw main rect
                self.fill_rect(x, y, width, height, color);
            }
            crate::core::gpu::shader::ShaderEffectType::Brightness => {
                let brightness = effect.parameters.brightness;
                let bright_r = (color.r as f32 * brightness).min(255.0) as u8;
                let bright_g = (color.g as f32 * brightness).min(255.0) as u8;
                let bright_b = (color.b as f32 * brightness).min(255.0) as u8;
                let bright_color = Color::rgba(bright_r, bright_g, bright_b, color.a);
                self.fill_rect(x, y, width, height, bright_color);
            }
            crate::core::gpu::shader::ShaderEffectType::Contrast => {
                let contrast = effect.parameters.contrast;
                let adjust = |c: u8| -> u8 {
                    let normalized = (c as f32 / 255.0 - 0.5) * contrast + 0.5;
                    (normalized * 255.0).max(0.0).min(255.0) as u8
                };
                let contrast_color = Color::rgba(
                    adjust(color.r),
                    adjust(color.g),
                    adjust(color.b),
                    color.a,
                );
                self.fill_rect(x, y, width, height, contrast_color);
            }
            crate::core::gpu::shader::ShaderEffectType::Desaturate => {
                let saturation = effect.parameters.saturation;
                let gray = (color.r as f32 * 0.299 + color.g as f32 * 0.587 + color.b as f32 * 0.114) as u8;
                let desat_r = (gray as f32 * (1.0 - saturation) + color.r as f32 * saturation) as u8;
                let desat_g = (gray as f32 * (1.0 - saturation) + color.g as f32 * saturation) as u8;
                let desat_b = (gray as f32 * (1.0 - saturation) + color.b as f32 * saturation) as u8;
                let desat_color = Color::rgba(desat_r, desat_g, desat_b, color.a);
                self.fill_rect(x, y, width, height, desat_color);
            }
            _ => {
                // Fallback to basic rendering
                self.fill_rect(x, y, width, height, color);
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
                        let radius_f = radius as f32;

                        if dist <= radius_f {
                            let px = corner_x + dx;
                            let py = corner_y + dy;

                            // Anti-aliasing at edges
                            if dist > radius_f - 1.5 {
                                let alpha = ((radius_f - dist) * 255.0).max(0.0).min(255.0) as u8;
                                let alpha = ((alpha as f32 / 255.0) * (color.a as f32 / 255.0) * 255.0) as u8;
                                let blended = Color::rgba(color.r, color.g, color.b, alpha);
                                self.blend_pixel(px, py, blended);
                            } else {
                                self.set_pixel(px, py, color);
                            }
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
                // Calculate position along gradient direction
                let nx = px as f32 / width as f32 - 0.5;
                let ny = py as f32 / height as f32 - 0.5;
                let t = (nx * cos_a + ny * sin_a + 0.5).max(0.0).min(1.0);

                // Interpolate colors
                let r = (start_color.r as f32 * (1.0 - t) + end_color.r as f32 * t) as u8;
                let g = (start_color.g as f32 * (1.0 - t) + end_color.g as f32 * t) as u8;
                let b = (start_color.b as f32 * (1.0 - t) + end_color.b as f32 * t) as u8;
                let a = (start_color.a as f32 * (1.0 - t) + end_color.a as f32 * t) as u8;
                let color = Color::rgba(r, g, b, a);

                self.set_pixel(x + px, y + py, color);
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

    pub fn draw_shadow(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        blur: i32,
        color: Color,
    ) {
        // Step 1: Create alpha-only texture for the shape
        let mut alpha_map = vec![0u8; (width * height) as usize];
        
        // Fill the shape area with full alpha
        for py in 0..height {
            for px in 0..width {
                let idx = (py * width + px) as usize;
                alpha_map[idx] = 255;
            }
        }
        
        // Step 2: Blur the alpha texture (box blur for performance)
        let mut blurred_alpha = alpha_map.clone();
        if blur > 0 {
            // Horizontal pass
            let mut temp = vec![0u8; (width * height) as usize];
            for py in 0..height {
                for px in 0..width {
                    let mut sum = 0u32;
                    let mut count = 0u32;
                    
                    for dx in -blur..=blur {
                        let sample_x = (px + dx).max(0).min(width - 1);
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
                    
                    for dy in -blur..=blur {
                        let sample_y = (py + dy).max(0).min(height - 1);
                        let idx = (sample_y * width + px) as usize;
                        sum += temp[idx] as u32;
                        count += 1;
                    }
                    
                    let idx = (py * width + px) as usize;
                    blurred_alpha[idx] = (sum / count) as u8;
                }
            }
        }
        
        // Step 3: Multiply shadow color by blurred alpha and composite
        let shadow_offset = blur / 2;
        for py in 0..height {
            for px in 0..width {
                let idx = (py * width + px) as usize;
                let alpha = blurred_alpha[idx];
                
                if alpha > 0 {
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

    /// Draw shadow with rounded corners support
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
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        
                        if dist <= radius as f32 {
                            alpha_map[idx] = 255;
                            break;
                        }
                    }
                }
            }
        }
        
        // Step 2: Blur the alpha texture (separable box blur)
        let mut blurred_alpha = alpha_map.clone();
        if blur > 0 {
            // Horizontal pass
            let mut temp = vec![0u8; (width * height) as usize];
            for py in 0..height {
                for px in 0..width {
                    let mut sum = 0u32;
                    let mut count = 0u32;
                    
                    for dx in -blur..=blur {
                        let sample_x = (px + dx).max(0).min(width - 1);
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
                    
                    for dy in -blur..=blur {
                        let sample_y = (py + dy).max(0).min(height - 1);
                        let idx = (sample_y * width + px) as usize;
                        sum += temp[idx] as u32;
                        count += 1;
                    }
                    
                    let idx = (py * width + px) as usize;
                    blurred_alpha[idx] = (sum / count) as u8;
                }
            }
        }
        
        // Step 3: Multiply shadow color by blurred alpha and composite
        let shadow_offset = blur / 2;
        for py in 0..height {
            for px in 0..width {
                let idx = (py * width + px) as usize;
                let alpha = blurred_alpha[idx];
                
                if alpha > 0 {
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
    }

}
