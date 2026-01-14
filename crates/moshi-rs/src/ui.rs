use crate::{canvas::Canvas, color::Color, text::TextRenderer};

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
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
}

impl Container {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            background: Color::BG_PRIMARY,
            children: Vec::new(),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }
}

impl Element for Container {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        canvas.fill_rect(self.rect.x, self.rect.y, self.rect.width, self.rect.height, self.background);
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
}

impl Element for Text {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        text_renderer.render(canvas, &self.text, self.x, self.y, self.size, self.color, &self.font);
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

pub struct Card {
    pub rect: Rect,
    pub background: Color,
    pub border_color: Option<Color>,
    pub border_width: i32,
    pub shadow: bool,
    pub shadow_blur: i32,
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
            children: Vec::new(),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
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

    pub fn child(mut self, element: impl Element + 'static) -> Self {
        self.children.push(Box::new(element));
        self
    }
}

impl Element for Card {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Draw shadow first (behind the card)
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

        // Draw card background
        canvas.fill_rect(
            self.rect.x,
            self.rect.y,
            self.rect.width,
            self.rect.height,
            self.background,
        );

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

pub fn card(x: i32, y: i32, width: i32, height: i32) -> Card {
    Card::new(Rect::new(x, y, width, height))
}

pub fn text(content: impl Into<String>, x: i32, y: i32) -> Text {
    Text::new(content, x, y)
}

pub fn vstack(x: i32, y: i32) -> VStack {
    VStack::new(x, y)
}

pub struct Titlebar {
    pub rect: Rect,
    pub title: String,
    pub background: Color,
    pub show_controls: bool,
}

impl Titlebar {
    pub fn new(width: i32, title: impl Into<String>) -> Self {
        Self {
            rect: Rect::new(0, 0, width, 32),
            title: title.into(),
            background: Color::BG_TERTIARY,
            show_controls: true,
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
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
        let icon_size = 14;
        let icon_y = y + (self.rect.height - icon_size) / 2;
        
        // Minimize icon (horizontal line) - no background
        let line_y = icon_y + icon_size / 2;
        let color = if hovered {
            Color::TEXT_PRIMARY
        } else {
            Color::TEXT_SECONDARY
        };
        canvas.fill_rect(x + 2, line_y, icon_size - 4, 2, color);
    }

    fn draw_maximize_button(&self, canvas: &mut Canvas, x: i32, y: i32, hovered: bool) {
        let icon_size = 14;
        let icon_y = y + (self.rect.height - icon_size) / 2;
        
        // Maximize icon (square outline) - no background
        let color = if hovered {
            Color::TEXT_PRIMARY
        } else {
            Color::TEXT_SECONDARY
        };
        canvas.draw_rect(x + 2, icon_y + 2, icon_size - 4, icon_size - 4, color, 1);
    }

    fn draw_close_button(&self, canvas: &mut Canvas, x: i32, y: i32, hovered: bool) {
        let icon_size = 14;
        let icon_y = y + (self.rect.height - icon_size) / 2;
        
        // Close icon (X) - no background, red on hover
        let color = if hovered {
            Color::rgba(220, 50, 50, 255)
        } else {
            Color::TEXT_SECONDARY
        };
        
        // Draw X using diagonal lines
        for i in 0..8 {
            // Top-left to bottom-right
            canvas.set_pixel(x + 2 + i, icon_y + 2 + i, color);
            // Top-right to bottom-left
            canvas.set_pixel(x + 9 - i, icon_y + 2 + i, color);
        }
    }
}

impl Element for Titlebar {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        // Draw titlebar background
        canvas.fill_rect(
            self.rect.x,
            self.rect.y,
            self.rect.width,
            self.rect.height,
            self.background,
        );

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
        let y_center = self.rect.y + (self.rect.height / 6) + (font_size as i32 / 3);

        // Render title text on the left
        text_renderer.render(
            canvas,
            &self.title,
            12,
            y_center,
            font_size,
            Color::TEXT_PRIMARY,
            "semibold",
        );

        // Draw window control buttons on the right
        if self.show_controls {
            let button_spacing = 8; // 8px spacing between buttons
            let right_margin = 12;
            
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
