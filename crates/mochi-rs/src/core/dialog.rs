use super::ui::{Element, Rect};
use crate::core::{canvas::Canvas, color::Color, text::TextRenderer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogButtonStyle {
    Primary,     // Blue, prominent
    Secondary,   // Gray, less prominent
    Destructive, // Red, for dangerous actions
}

#[derive(Debug, Clone)]
pub struct DialogButton {
    pub label: String,
    pub style: DialogButtonStyle,
    pub rect: Rect,
}

impl DialogButton {
    pub fn new(label: impl Into<String>, style: DialogButtonStyle) -> Self {
        Self {
            label: label.into(),
            style,
            rect: Rect::new(0, 0, 0, 0), // Will be set by dialog layout
        }
    }

    fn background_color(&self) -> Color {
        match self.style {
            DialogButtonStyle::Primary => Color::rgb(0, 122, 255), // macOS blue
            DialogButtonStyle::Secondary => Color::rgb(200, 200, 200),
            DialogButtonStyle::Destructive => Color::rgb(255, 59, 48),
        }
    }

    fn text_color(&self) -> Color {
        match self.style {
            DialogButtonStyle::Primary => Color::rgb(255, 255, 255),
            DialogButtonStyle::Secondary => Color::rgb(60, 60, 60),
            DialogButtonStyle::Destructive => Color::rgb(255, 255, 255),
        }
    }
}

pub struct Dialog {
    pub rect: Rect,
    pub title: String,
    pub message: String,
    pub icon: Option<String>, // Optional icon/emoji
    pub buttons: Vec<DialogButton>,
    background: Color,
    corner_radius: i32,
    padding: i32,
}

impl Dialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rect: Rect::new(0, 0, 380, 310), // Adjusted height for better proportions
            title: title.into(),
            message: message.into(),
            icon: None,
            buttons: Vec::new(),
            background: Color::rgb(242, 242, 247), // macOS light gray
            corner_radius: 14,
            padding: 20,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn button(mut self, button: DialogButton) -> Self {
        self.buttons.push(button);
        self
    }

    pub fn centered(mut self, canvas_width: i32, canvas_height: i32) -> Self {
        self.rect.x = (canvas_width - self.rect.width) / 2;
        self.rect.y = (canvas_height - self.rect.height) / 2;
        self
    }

    fn draw_rounded_rect(
        &self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
    ) {
        let radius = self.corner_radius;

        // Fill main body
        canvas.fill_rect(x + radius, y, width - radius * 2, height, color);
        canvas.fill_rect(x, y + radius, width, height - radius * 2, color);

        // Draw corners with anti-aliasing for smoother appearance
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
                            let px = if corner_x == x {
                                corner_x + dx
                            } else {
                                corner_x + dx
                            };
                            let py = if corner_y == y {
                                corner_y + dy
                            } else {
                                corner_y + dy
                            };

                            // Anti-aliasing: blend at edges
                            if dist > radius_f - 1.5 {
                                let alpha = ((radius_f - dist) * 255.0).max(0.0).min(255.0) as u8;
                                let blended = Color::rgba(color.r, color.g, color.b, alpha);
                                canvas.blend_pixel(px, py, blended);
                            } else {
                                canvas.set_pixel(px, py, color);
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_button(
        &self,
        canvas: &mut Canvas,
        text_renderer: &TextRenderer,
        button: &DialogButton,
    ) {
        let rect = &button.rect;
        let bg_color = button.background_color();
        let text_color = button.text_color();

        // Draw rounded button background with anti-aliasing
        let button_radius = 10;
        let radius = button_radius;

        // Fill main body
        canvas.fill_rect(
            rect.x + radius,
            rect.y,
            rect.width - radius * 2,
            rect.height,
            bg_color,
        );
        canvas.fill_rect(
            rect.x,
            rect.y + radius,
            rect.width,
            rect.height - radius * 2,
            bg_color,
        );

        // Draw corners with anti-aliasing
        for corner_x in [rect.x, rect.x + rect.width - radius] {
            for corner_y in [rect.y, rect.y + rect.height - radius] {
                for dy in 0..radius {
                    for dx in 0..radius {
                        let center_x = if corner_x == rect.x { radius } else { 0 };
                        let center_y = if corner_y == rect.y { radius } else { 0 };

                        let dist = (((dx - center_x) * (dx - center_x)
                            + (dy - center_y) * (dy - center_y))
                            as f32)
                            .sqrt();
                        let radius_f = radius as f32;

                        if dist <= radius_f {
                            let px = if corner_x == rect.x {
                                corner_x + dx
                            } else {
                                corner_x + dx
                            };
                            let py = if corner_y == rect.y {
                                corner_y + dy
                            } else {
                                corner_y + dy
                            };

                            // Anti-aliasing: blend at edges
                            if dist > radius_f - 1.5 {
                                let alpha = ((radius_f - dist) * 255.0).max(0.0).min(255.0) as u8;
                                let blended =
                                    Color::rgba(bg_color.r, bg_color.g, bg_color.b, alpha);
                                canvas.blend_pixel(px, py, blended);
                            } else {
                                canvas.set_pixel(px, py, bg_color);
                            }
                        }
                    }
                }
            }
        }

        // Draw button text (centered)
        let text_size = 16.0;
        let text_width = (button.label.len() as f32 * text_size * 0.5) as i32;
        let text_x = rect.x + (rect.width - text_width) / 2;
        let text_y = rect.y + (rect.height / 2) + 5; // Vertically centered

        text_renderer.render(
            canvas,
            &button.label,
            text_x,
            text_y,
            text_size,
            text_color,
            "semibold",
        );
    }
}

impl Element for Dialog {
    fn render(&self, canvas: &mut Canvas, text_renderer: &TextRenderer) {
        let x = self.rect.x;
        let y = self.rect.y;
        let width = self.rect.width;
        let height = self.rect.height;

        // Draw shadow (simple offset)
        let shadow_color = Color::rgba(0, 0, 0, 50);
        self.draw_rounded_rect(canvas, x + 3, y + 3, width, height, shadow_color);

        // Draw dialog background
        self.draw_rounded_rect(canvas, x, y, width, height, self.background);

        let mut current_y = y + self.padding;

        // Draw icon if present (as emoji/text for now)
        if let Some(ref icon) = self.icon {
            let icon_size = 48.0;
            let icon_x = x + (width / 2) - 24;
            text_renderer.render(
                canvas,
                icon,
                icon_x,
                current_y + 10,
                icon_size,
                Color::rgb(0, 122, 255),
                "regular",
            );
            current_y += 65;
        }

        // Draw title
        let title_size = 17.0;
        let title_width = (self.title.len() as f32 * title_size * 0.5) as i32;
        let title_x = x + (width - title_width) / 2;
        text_renderer.render(
            canvas,
            &self.title,
            title_x,
            current_y,
            title_size,
            Color::rgb(0, 0, 0),
            "semibold",
        );
        current_y += 30;

        // Draw message (word-wrapped, centered)
        let message_size = 13.0;
        let message_color = Color::rgb(100, 100, 100);
        let max_line_width = width - self.padding * 3;

        // Simple word wrapping
        let words: Vec<&str> = self.message.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let test_width = (test_line.len() as f32 * message_size * 0.5) as i32;
            if test_width > max_line_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // Render centered lines
        for line in &lines {
            let line_width = (line.len() as f32 * message_size * 0.5) as i32;
            let line_x = x + (width - line_width) / 2;
            text_renderer.render(
                canvas,
                line,
                line_x,
                current_y,
                message_size,
                message_color,
                "regular",
            );
            current_y += 20;
        }

        current_y += 15;

        // Draw buttons (stacked vertically, macOS style)
        let button_height = 40;
        let button_spacing = 10;
        let button_width = width - self.padding * 2;

        for (i, button) in self.buttons.iter().enumerate() {
            let button_y = current_y + (i as i32 * (button_height + button_spacing));
            let mut btn = button.clone();
            btn.rect = Rect::new(x + self.padding, button_y, button_width, button_height);
            self.draw_button(canvas, text_renderer, &btn);
        }
    }

    fn bounds(&self) -> Rect {
        self.rect.clone()
    }
}

// Convenience function
pub fn dialog(title: impl Into<String>, message: impl Into<String>) -> Dialog {
    Dialog::new(title, message)
}
