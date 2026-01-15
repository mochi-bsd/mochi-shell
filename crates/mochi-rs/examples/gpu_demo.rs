use mochi::{Canvas, Color, GpuRenderer, ShaderEffect, SoftwareBackend, Window, WindowConfig};
use glam::Vec2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WindowConfig {
        title: "Mochi GPU Shader Effects Demo".to_string(),
        width: 1200,
        height: 800,
        min_width: Some(800),
        min_height: Some(600),
        decorations: false,
    };

    let mut window = Window::new(config)?;

    window.on_draw(move |canvas: &mut Canvas| {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;

        // Clear background
        canvas.clear(Color::rgb(20, 20, 25));

        // Title
        canvas.fill_rect(0, 0, width, 60, Color::rgb(30, 30, 35));

        // Demo grid layout
        let margin = 40;
        let spacing = 20;
        let card_width = (width - margin * 2 - spacing * 2) / 3;
        let card_height = 200;

        // Row 1: Basic effects
        let y1 = 80;

        // Rounded rectangle
        canvas.fill_rounded_rect(
            margin,
            y1,
            card_width,
            card_height,
            16.0,
            Color::rgb(100, 50, 200),
        );

        // Gradient
        canvas.fill_gradient_rect(
            margin + card_width + spacing,
            y1,
            card_width,
            card_height,
            Color::rgb(255, 100, 100),
            Color::rgb(100, 100, 255),
            45.0,
        );

        // Glow effect
        if canvas.has_gpu() {
            let glow = ShaderEffect::glow(1.5);
            canvas.fill_rect_with_effect(
                margin + (card_width + spacing) * 2,
                y1,
                card_width,
                card_height,
                Color::rgb(100, 255, 100),
                &glow,
            );
        } else {
            canvas.fill_rect(
                margin + (card_width + spacing) * 2,
                y1,
                card_width,
                card_height,
                Color::rgb(100, 255, 100),
            );
        }

        // Row 2: Color effects
        let y2 = y1 + card_height + spacing;

        // Brightness
        if canvas.has_gpu() {
            let bright = ShaderEffect::brightness(1.5);
            canvas.fill_rect_with_effect(
                margin,
                y2,
                card_width,
                card_height,
                Color::rgb(150, 150, 150),
                &bright,
            );
        } else {
            canvas.fill_rect(margin, y2, card_width, card_height, Color::rgb(200, 200, 200));
        }

        // Contrast
        if canvas.has_gpu() {
            let contrast = ShaderEffect::contrast(2.0);
            canvas.fill_rect_with_effect(
                margin + card_width + spacing,
                y2,
                card_width,
                card_height,
                Color::rgb(150, 150, 150),
                &contrast,
            );
        } else {
            canvas.fill_rect(
                margin + card_width + spacing,
                y2,
                card_width,
                card_height,
                Color::rgb(150, 150, 150),
            );
        }

        // Desaturate
        if canvas.has_gpu() {
            let desat = ShaderEffect::desaturate(0.8);
            canvas.fill_rect_with_effect(
                margin + (card_width + spacing) * 2,
                y2,
                card_width,
                card_height,
                Color::rgb(255, 100, 150),
                &desat,
            );
        } else {
            canvas.fill_rect(
                margin + (card_width + spacing) * 2,
                y2,
                card_width,
                card_height,
                Color::rgb(180, 120, 140),
            );
        }

        // Row 3: Advanced effects
        let y3 = y2 + card_height + spacing;

        // Blur
        if canvas.has_gpu() {
            let blur = ShaderEffect::blur(8.0);
            canvas.fill_rect_with_effect(
                margin,
                y3,
                card_width,
                card_height,
                Color::rgb(255, 200, 100),
                &blur,
            );
        } else {
            canvas.fill_rect(margin, y3, card_width, card_height, Color::rgb(255, 200, 100));
        }

        // Shadow
        if canvas.has_gpu() {
            let shadow = ShaderEffect::shadow(Vec2::new(4.0, 4.0), 8.0);
            canvas.fill_rect_with_effect(
                margin + card_width + spacing,
                y3,
                card_width,
                card_height,
                Color::rgb(100, 200, 255),
                &shadow,
            );
        } else {
            canvas.draw_shadow(
                margin + card_width + spacing,
                y3,
                card_width,
                card_height,
                8,
                Color::rgba(0, 0, 0, 100),
            );
            canvas.fill_rect(
                margin + card_width + spacing,
                y3,
                card_width,
                card_height,
                Color::rgb(100, 200, 255),
            );
        }

        // Combined rounded + gradient
        canvas.fill_rounded_rect(
            margin + (card_width + spacing) * 2,
            y3,
            card_width,
            card_height,
            20.0,
            Color::rgb(200, 100, 255),
        );

        // Status text
        let status = if canvas.has_gpu() {
            "GPU Accelerated ✓"
        } else {
            "Software Rendering"
        };
        
        // Simple text rendering (you'd use TextRenderer in real app)
        canvas.fill_rect(width - 200, 15, 180, 30, Color::rgba(50, 50, 55, 200));
    });

    window.run()
}
