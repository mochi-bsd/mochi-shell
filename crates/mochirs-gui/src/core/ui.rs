use crate::core::{canvas::Canvas, color::Color, text::TextRenderer};

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

pub trait Element {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer);
    fn bounds(&self) -> Rect;
}

pub struct Container {
    pub rect: Rect,
    pub background: Color,
    pub children: Vec<Box<dyn Element>>,
    // shader_effect removed - using software rendering only
    pub corner_radius: Option<f32>,
}

impl Container {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            background: Color::BG_PRIMARY,
            children: Vec::new(),
            // shader_effect: None,
            corner_radius: None,
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }


    pub fn blur(mut self, radius: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::blur(radius));
        self
    }

    pub fn glow(mut self, intensity: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::glow(intensity));
        self
    }

    pub fn brightness(mut self, value: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::brightness(value));
        self
    }

    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }
}

impl Element for Container {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        if let Some(radius) = self.corner_radius {
            if false { // shader effects disabled
            } else {
                canvas.fill_rounded_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    radius,
                    self.background,
                );
            }
        } else if false { // shader effects disabled
        } else {
            canvas.fill_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.background,
            );
        }

        for child in &self.children {
            child.render(canvas, text_renderer);
        }
    }

    fn bounds(&self) -> Rect {
        self.rect.clone()
    }
}

pub struct Text {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub size: f32,
    pub color: Color,
    pub font: String,
    pub shadow: bool,
    pub shadow_offset: (i32, i32),
    pub shadow_color: Color,
    pub shadow_blur: i32,
}

impl Text {
    pub fn new(text: impl Into<String>, x: i32, y: i32) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            size: 16.0,
            color: Color::TEXT_PRIMARY,
            font: "regular".to_string(),
            shadow: false,
            shadow_offset: (2, 2),
            shadow_color: Color::rgba(0, 0, 0, 128),
            shadow_blur: 2,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.font = font.into();
        self
    }

    pub fn shadow(mut self, enabled: bool) -> Self {
        self.shadow = enabled;
        self
    }

    pub fn shadow_offset(mut self, x: i32, y: i32) -> Self {
        self.shadow_offset = (x, y);
        self
    }

    pub fn shadow_color(mut self, color: Color) -> Self {
        self.shadow_color = color;
        self
    }

    pub fn shadow_blur(mut self, blur: i32) -> Self {
        self.shadow_blur = blur;
        self
    }
}

impl Element for Text {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Render shadow first (behind the text) if enabled
        if self.shadow && self.shadow_blur > 0 {
            // Proper box blur with horizontal and vertical passes
            let blur_radius = self.shadow_blur.min(4); // Limit to 4 for performance
            
            // Box blur: uniform weight distribution
            let weight = 1.0 / ((blur_radius * 2 + 1) as f32);
            
            // Horizontal pass
            for dx in -blur_radius..=blur_radius {
                let shadow_alpha = ((self.shadow_color.a as f32 * weight * 0.5).min(50.0)) as u8;
                
                if shadow_alpha > 1 {
                    let blur_color = Color::rgba(
                        self.shadow_color.r,
                        self.shadow_color.g,
                        self.shadow_color.b,
                        shadow_alpha,
                    );
                    
                    text_renderer.render(
                        canvas,
                        &self.text,
                        self.x + self.shadow_offset.0 + dx,
                        self.y + self.shadow_offset.1,
                        self.size,
                        blur_color,
                        &self.font,
                    );
                }
            }
            
            // Vertical pass
            for dy in -blur_radius..=blur_radius {
                if dy == 0 { continue; } // Skip center, already done in horizontal
                
                let shadow_alpha = ((self.shadow_color.a as f32 * weight * 0.5).min(50.0)) as u8;
                
                if shadow_alpha > 1 {
                    let blur_color = Color::rgba(
                        self.shadow_color.r,
                        self.shadow_color.g,
                        self.shadow_color.b,
                        shadow_alpha,
                    );
                    
                    text_renderer.render(
                        canvas,
                        &self.text,
                        self.x + self.shadow_offset.0,
                        self.y + self.shadow_offset.1 + dy,
                        self.size,
                        blur_color,
                        &self.font,
                    );
                }
            }
        } else if self.shadow {
            // Simple shadow without blur
            text_renderer.render(
                canvas,
                &self.text,
                self.x + self.shadow_offset.0,
                self.y + self.shadow_offset.1,
                self.size,
                self.shadow_color,
                &self.font,
            );
        }
        
        // ALWAYS render main text on top - this is the foreground layer
        text_renderer.render(
            canvas, &self.text, self.x, self.y, self.size, self.color, &self.font,
        );
    }

    fn bounds(&self) -> Rect {
        // Approximate bounds based on font size
        // For more accurate bounds, we'd need access to text_renderer
        let approx_width = (self.text.len() as f32 * self.size * 0.6) as i32;
        let approx_height = (self.size * 1.2) as i32;
        Rect::new(self.x, self.y, approx_width, approx_height)
    }
}

pub struct VStack {
    pub x: i32,
    pub y: i32,
    pub spacing: i32,
    pub children: Vec<Box<dyn Element>>,
}

impl VStack {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            spacing: 10,
            children: Vec::new(),
        }
    }

    pub fn spacing(mut self, spacing: i32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }
}

impl Element for VStack {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        for child in &self.children {
            child.render(canvas, text_renderer);
        }
    }

    fn bounds(&self) -> Rect {
        let mut total_height = 0;
        let mut max_width = 0;

        for (i, child) in self.children.iter().enumerate() {
            let bounds = child.bounds();
            total_height += bounds.height;
            if i > 0 {
                total_height += self.spacing;
            }
            max_width = max_width.max(bounds.width);
        }

        Rect::new(self.x, self.y, max_width, total_height)
    }
}

// Div - A flexible container element (like HTML div)
pub struct Div {
    pub rect: Rect,
    pub background: Color,
    pub border_color: Option<Color>,
    pub border_width: i32,
    pub shadow: bool,
    pub shadow_blur: i32,
    pub corner_radius: f32,
    pub gradient: Option<(Color, f32)>,
    pub children: Vec<Box<dyn Element>>,
}

impl Div {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            background: Color::TRANSPARENT,
            border_color: None,
            border_width: 0,
            shadow: false,
            shadow_blur: 0,
            corner_radius: 0.0,
            gradient: None,
            children: Vec::new(),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn gradient(mut self, end_color: Color, angle: f32) -> Self {
        self.gradient = Some((end_color, angle));
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn border(mut self, color: Color, width: i32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    pub fn shadow(mut self, enabled: bool) -> Self {
        self.shadow = enabled;
        self
    }

    pub fn shadow_blur(mut self, blur: i32) -> Self {
        self.shadow_blur = blur;
        self
    }

    pub fn frame(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.rect = Rect::new(x, y, width, height);
        self
    }

    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }
}

impl Element for Div {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Draw shadow first (behind the div) with proper blur
        if self.shadow && self.shadow_blur > 0 {
            if self.corner_radius > 0.0 {
                canvas.draw_rounded_shadow(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.corner_radius,
                    self.shadow_blur,
                    Color::rgba(0, 0, 0, 80),
                );
            } else {
                canvas.draw_shadow(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.shadow_blur,
                    Color::rgba(0, 0, 0, 80),
                );
            }
        }

        // Draw div background with effects
        if let Some((end_color, angle)) = self.gradient {
            if self.corner_radius > 0.0 {
                // For gradients with rounded corners, draw gradient then mask
                canvas.fill_gradient_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.background,
                    end_color,
                    angle,
                );
                // Redraw rounded rect to clip corners
                canvas.fill_rounded_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.corner_radius,
                    Color::TRANSPARENT,
                );
            } else {
                canvas.fill_gradient_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.background,
                    end_color,
                    angle,
                );
            }
        } else if self.background.a > 0 {
            // Only draw background if not transparent
            canvas.fill_rounded_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.corner_radius,
                self.background,
            );
        }

        // Draw border if enabled
        if let Some(border_color) = self.border_color {
            canvas.draw_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                border_color,
                self.border_width,
            );
        }

        // Render children
        for child in &self.children {
            child.render(canvas, text_renderer);
        }
    }

    fn bounds(&self) -> Rect {
        self.rect.clone()
    }
}

// Card - Deprecated, use Div instead
// Kept for backward compatibility
pub struct Card {
    pub rect: Rect,
    pub background: Color,
    pub border_color: Option<Color>,
    pub border_width: i32,
    pub shadow: bool,
    pub shadow_blur: i32,
    pub corner_radius: f32,
    // shader_effect removed - using software rendering only
    pub gradient: Option<(Color, f32)>,
    pub children: Vec<Box<dyn Element>>,
}

impl Card {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            background: Color::BG_SECONDARY,
            border_color: Some(Color::rgba(60, 60, 70, 255)),
            border_width: 1,
            shadow: true,
            shadow_blur: 8,
            corner_radius: 12.0,
            // shader_effect: None,
            gradient: None,
            children: Vec::new(),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn gradient(mut self, end_color: Color, angle: f32) -> Self {
        self.gradient = Some((end_color, angle));
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn border(mut self, color: Color, width: i32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border_color = None;
        self
    }

    pub fn shadow(mut self, enabled: bool) -> Self {
        self.shadow = enabled;
        self
    }

    pub fn shadow_blur(mut self, blur: i32) -> Self {
        self.shadow_blur = blur;
        self
    }


    pub fn blur(mut self, radius: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::blur(radius));
        self
    }

    pub fn glow(mut self, intensity: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::glow(intensity));
        self
    }

    pub fn brightness(mut self, value: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::brightness(value));
        self
    }

    pub fn contrast(mut self, value: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::contrast(value));
        self
    }

    pub fn desaturate(mut self, amount: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::desaturate(amount));
        self
    }

    pub fn with_shader<F>(self, shader_fn: F) -> ShaderCard
    where
        F: Fn(&mut Canvas, i32, i32, i32, i32) + 'static,
    {
        ShaderCard {
            rect: self.rect,
            background: self.background,
            border_color: self.border_color,
            border_width: self.border_width,
            shadow: self.shadow,
            shadow_blur: self.shadow_blur,
            corner_radius: self.corner_radius,
            children: self.children,
            shader_fn: Box::new(shader_fn),
        }
    }

    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }

    // Helper to apply rounded corner mask by clearing corners
    fn apply_rounded_mask(&self, canvas: &mut Canvas) {
        let radius = self.corner_radius as i32;
        let x = self.rect.x;
        let y = self.rect.y;
        let width = self.rect.width;
        let height = self.rect.height;

        // Clear corners to create rounded effect
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

                        // Clear pixels outside the rounded corner
                        if dist > radius_f {
                            let px = corner_x + dx;
                            let py = corner_y + dy;
                            canvas.set_pixel(px, py, Color::BG_PRIMARY);
                        }
                    }
                }
            }
        }
    }
}

impl Element for Card {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Draw shadow first (behind the card) with proper blur
        if self.shadow {
            if self.corner_radius > 0.0 {
                canvas.draw_rounded_shadow(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.corner_radius,
                    self.shadow_blur,
                    Color::rgba(0, 0, 0, 80),
                );
            } else {
                canvas.draw_shadow(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.shadow_blur,
                    Color::rgba(0, 0, 0, 80),
                );
            }
        }

        // Draw card background with effects
        if let Some((end_color, angle)) = self.gradient {
            if self.corner_radius > 0.0 {
                // For gradients with rounded corners, draw gradient then mask
                canvas.fill_gradient_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.background,
                    end_color,
                    angle,
                );
                self.apply_rounded_mask(canvas);
            } else {
                canvas.fill_gradient_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.background,
                    end_color,
                    angle,
                );
            }
        } else if false { // shader effects disabled
            if self.corner_radius > 0.0 {
                // For effects with rounded corners:
                // 1. Draw the effect (may go outside bounds)
                // 2. Draw rounded rect on top to clip
                // 3. Apply mask to clean up corners
                // Redraw the rounded rect to clip the effect
                canvas.fill_rounded_rect(
                    self.rect.x,
                    self.rect.y,
                    self.rect.width,
                    self.rect.height,
                    self.corner_radius,
                    self.background,
                );
            } else {
            }
        } else {
            canvas.fill_rounded_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.corner_radius,
                self.background,
            );
        }

        // Draw border if enabled
        if let Some(border_color) = self.border_color {
            canvas.draw_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                border_color,
                self.border_width,
            );
        }

        // Render children
        for child in &self.children {
            child.render(canvas, text_renderer);
        }
    }

    fn bounds(&self) -> Rect {
        self.rect.clone()
    }
}

// Builder functions for ergonomic API
pub fn container(x: i32, y: i32, width: i32, height: i32) -> Container {
    Container::new(Rect::new(x, y, width, height))
}

// Constructor functions
pub fn div(x: i32, y: i32, width: i32, height: i32) -> Div {
    Div::new(Rect::new(x, y, width, height))
}

pub fn card(x: i32, y: i32, width: i32, height: i32) -> Card {
    Card::new(Rect::new(x, y, width, height))
}

pub fn text(content: impl Into<String>, x: i32, y: i32) -> Text {
    Text::new(content, x, y)
}

pub fn vstack(x: i32, y: i32) -> VStack {
    VStack::new(x, y)
}

// SVG icon assets embedded at compile time
const SVG_MINIMIZE: &str = include_str!("../assets/ui_minimize.svg");
const SVG_MAXIMIZE: &str = include_str!("../assets/ui_maximize.svg");
const SVG_RESTORE: &str = include_str!("../assets/ui_restore.svg");
const SVG_QUIT: &str = include_str!("../assets/ui_quit.svg");

pub struct Titlebar {
    pub rect: Rect,
    pub title: String,
    pub background: Color,
    pub show_controls: bool,
    // shader_effect removed - using software rendering only
    pub gradient: Option<(Color, f32)>,
}

impl Titlebar {
    pub fn new(width: i32, title: impl Into<String>) -> Self {
        Self {
            rect: Rect::new(0, 0, width, 40),
            title: title.into(),
            background: Color::BG_TERTIARY,
            show_controls: true,
            // shader_effect: None,
            gradient: None,
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn gradient(mut self, end_color: Color, angle: f32) -> Self {
        self.gradient = Some((end_color, angle));
        self
    }


    pub fn blur(mut self, radius: f32) -> Self {
        // self.shader_effect = Some(ShaderEffect::blur(radius));
        self
    }

    pub fn height(mut self, height: i32) -> Self {
        self.rect.height = height;
        self
    }

    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    fn draw_minimize_button(&self, canvas: &mut Canvas, x: i32, y: i32, hovered: bool) {
        let color = if hovered {
            Color::TEXT_PRIMARY
        } else {
            Color::TEXT_SECONDARY
        };

        // Center the 16x16 icon in the button area
        let icon_size = 10;
        let icon_x = x + (32 - icon_size) / 2;
        let icon_y = y + (self.rect.height - icon_size) / 2;

        Self::render_svg(
            canvas,
            SVG_MINIMIZE,
            icon_x,
            icon_y,
            icon_size as u32,
            icon_size as u32,
            color,
        );
    }

    fn draw_maximize_button(&self, canvas: &mut Canvas, x: i32, y: i32, hovered: bool) {
        let color = if hovered {
            Color::TEXT_PRIMARY
        } else {
            Color::TEXT_SECONDARY
        };

        // Center the 16x16 icon in the button area
        let icon_size = 10;
        let icon_x = x + (32 - icon_size) / 2;
        let icon_y = y + (self.rect.height - icon_size) / 2;

        Self::render_svg(
            canvas,
            SVG_MAXIMIZE,
            icon_x,
            icon_y,
            icon_size as u32,
            icon_size as u32,
            color,
        );
    }

    fn draw_close_button(&self, canvas: &mut Canvas, x: i32, y: i32, hovered: bool) {
        let color = if hovered {
            Color::rgba(220, 50, 50, 255)
        } else {
            Color::TEXT_SECONDARY
        };

        // Center the 16x16 icon in the button area
        let icon_size = 10;
        let icon_x = x + (32 - icon_size) / 2;
        let icon_y = y + (self.rect.height - icon_size) / 2;

        Self::render_svg(
            canvas,
            SVG_QUIT,
            icon_x,
            icon_y,
            icon_size as u32,
            icon_size as u32,
            color,
        );
    }

    fn render_svg(
        canvas: &mut Canvas,
        svg_data: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Color,
    ) {
        use resvg::usvg;
        use tiny_skia::{Pixmap, Transform};

        // Parse SVG
        let opt = usvg::Options::default();
        let tree = match usvg::Tree::from_str(svg_data, &opt) {
            Ok(tree) => tree,
            Err(_) => return,
        };

        // Create pixmap for rendering at higher resolution for better quality
        let scale_factor = 2.0; // Render at 2x for better antialiasing
        let render_width = (width as f32 * scale_factor) as u32;
        let render_height = (height as f32 * scale_factor) as u32;

        let mut pixmap = match Pixmap::new(render_width, render_height) {
            Some(p) => p,
            None => return,
        };

        // Calculate scale to fit SVG into target size
        let svg_size = tree.size();
        let scale_x = render_width as f32 / svg_size.width();
        let scale_y = render_height as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // Create transform for scaling
        let transform = Transform::from_scale(scale, scale);

        // Render SVG to pixmap with scaling
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Downsample and copy to canvas with proper alpha blending
        let pixels = pixmap.data();
        for py in 0..height {
            for px in 0..width {
                // Sample from the higher resolution pixmap (simple box filter)
                let src_x = (px as f32 * scale_factor) as u32;
                let src_y = (py as f32 * scale_factor) as u32;

                // Average 2x2 samples for antialiasing
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;

                for dy in 0..2 {
                    for dx in 0..2 {
                        let sample_x = src_x + dx;
                        let sample_y = src_y + dy;
                        if sample_x < render_width && sample_y < render_height {
                            let idx = ((sample_y * render_width + sample_x) * 4) as usize;
                            if idx + 3 < pixels.len() {
                                r_sum += pixels[idx] as u32;
                                g_sum += pixels[idx + 1] as u32;
                                b_sum += pixels[idx + 2] as u32;
                                a_sum += pixels[idx + 3] as u32;
                                count += 1;
                            }
                        }
                    }
                }

                if count > 0 && a_sum > 0 {
                    let avg_alpha = (a_sum / count) as u8;

                    // Apply color tint to white SVG
                    let tinted = Color::rgba(color.r, color.g, color.b, avg_alpha);

                    // Use blend_pixel for proper alpha compositing
                    canvas.blend_pixel(x + px as i32, y + py as i32, tinted);
                }
            }
        }
    }
}

impl Element for Titlebar {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Draw titlebar background with effects
        if let Some((end_color, angle)) = self.gradient {
            canvas.fill_gradient_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.background,
                end_color,
                angle,
            );
        } else if false { // shader effects disabled
        } else {
            canvas.fill_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.background,
            );
        }

        // Draw bottom border
        canvas.fill_rect(
            self.rect.x,
            self.rect.y + self.rect.height - 1,
            self.rect.width,
            1,
            Color::rgba(50, 50, 60, 255),
        );

        // Calculate vertical center position for text
        // For proper centering, we need to account for the baseline
        let font_size = 13.0;
        let y_center = self.rect.y + (self.rect.height / 4) + (font_size as i32 / 4);

        // Render title text on the left
        text_renderer.render(
            canvas,
            &self.title,
            12,
            y_center,
            font_size,
            Color::TEXT_PRIMARY,
            "bold",
        );

        // Draw window control buttons on the right
        if self.show_controls {
            let button_spacing = 24; // 8px spacing between buttons
            let right_margin = 24;

            // Close button (rightmost)
            let close_x = self.rect.x + self.rect.width - right_margin - 12;
            self.draw_close_button(canvas, close_x, self.rect.y, false);

            // Maximize button
            let maximize_x = close_x - 12 - button_spacing;
            self.draw_maximize_button(canvas, maximize_x, self.rect.y, false);

            // Minimize button
            let minimize_x = maximize_x - 12 - button_spacing;
            self.draw_minimize_button(canvas, minimize_x, self.rect.y, false);
        }
    }

    fn bounds(&self) -> Rect {
        self.rect.clone()
    }
}

pub fn titlebar(width: i32, title: impl Into<String>) -> Titlebar {
    Titlebar::new(width, title)
}

// Positioning trait implementations for ergonomic layout
impl Container {
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.rect.x = x;
        self.rect.y = y;
        self
    }

    pub fn size_to(mut self, width: i32, height: i32) -> Self {
        self.rect.width = width;
        self.rect.height = height;
        self
    }

    pub fn frame(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.rect = Rect::new(x, y, width, height);
        self
    }
}

impl Card {
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.rect.x = x;
        self.rect.y = y;
        self
    }

    pub fn size_to(mut self, width: i32, height: i32) -> Self {
        self.rect.width = width;
        self.rect.height = height;
        self
    }

    pub fn frame(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.rect = Rect::new(x, y, width, height);
        self
    }
}

impl Text {
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

impl VStack {
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}


// ShaderCard - Card with custom shader support
pub struct ShaderCard {
    pub rect: Rect,
    pub background: Color,
    pub border_color: Option<Color>,
    pub border_width: i32,
    pub shadow: bool,
    pub shadow_blur: i32,
    pub corner_radius: f32,
    pub children: Vec<Box<dyn Element>>,
    pub shader_fn: Box<dyn Fn(&mut Canvas, i32, i32, i32, i32)>,
}

impl ShaderCard {
    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }
}

impl Element for ShaderCard {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Draw shadow first
        if self.shadow {
            canvas.draw_shadow(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.shadow_blur,
                Color::rgba(0, 0, 0, 80),
            );
        }

        // Execute custom shader
        (self.shader_fn)(canvas, self.rect.x, self.rect.y, self.rect.width, self.rect.height);

        // Draw border if enabled
        if let Some(border_color) = self.border_color {
            canvas.draw_rect(
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                border_color,
                self.border_width,
            );
        }

        // Render children
        for child in &self.children {
            child.render(canvas, text_renderer);
        }
    }

    fn bounds(&self) -> Rect {
        self.rect.clone()
    }
}
