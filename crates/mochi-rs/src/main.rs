use mochi::{card, container, text, titlebar, vstack};
use mochi::{Canvas, Color, Element, ShaderEffect, TextRenderer, Window, WindowConfig};
use mochi::{BoxBlurShader, GaussianBlurShader, DropShadowShader, Vec2, Vec4};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load fonts
    let mut text_renderer = TextRenderer::new();
    let inter_regular = include_bytes!("../../../fs/library/shared/fonts/Inter-Regular.ttf");
    text_renderer.load_font("regular", inter_regular)?;
    let inter_semibold = include_bytes!("../../../fs/library/shared/fonts/Inter-SemiBold.ttf");
    text_renderer.load_font("semibold", inter_semibold)?;
    let inter_bold = include_bytes!("../../../fs/library/shared/fonts/Inter-Bold.ttf");
    text_renderer.load_font("bold", inter_bold)?;
    // Create window
    let config = WindowConfig {
        title: "Mochi Desktop Environment - GPU Accelerated".to_string(),
        width: 1400,
        height: 900,
        min_width: Some(800),
        min_height: Some(600),
        decorations: false,
        prefer_gpu: true, // Try to use GPU rendering
    };

    let mut window = Window::new(config)?;

    // Set up draw callback with RSX-like declarative UI
    window.on_draw(move |canvas: &mut Canvas| {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;

        // Detect renderer backend
        let renderer_type = canvas.get_renderer_type();
        let device_name = canvas.get_device_name();
        let titlebar_text = format!("Mochi Desktop - {} ({})", renderer_type, device_name);

        // Layout calculations
        let margin = 40;
        let card_width = width - (margin * 2);
        let card_height = 380;
        let demo_card_width = (card_width - 30) / 4;
        let demo_card_height = 140;

        // Build UI tree with RSX-like style
        let ui = container(0, 0, 0, 0)
            .frame(0, 0, width, height)
            .background(Color::BG_PRIMARY)
            .child(
                // Titlebar with gradient effect and renderer info
                titlebar(width, titlebar_text)
                    .gradient(Color::rgb(40, 40, 50), 90.0),
            )
            .child(
                // Main content card with glow effect
                card(0, 0, 0, 0)
                    .frame(margin, 60, card_width, card_height)
                    .rounded(16.0)
                    .shadow(true)
                    .shadow_blur(12)
                    .glow(0.3)
                    .child(
                        text("Welcome to Mochi", 0, 0)
                            .at(60, 80)
                            .size(42.0)
                            .color(Color::TEXT_PRIMARY)
                            .font("semibold"),
                    )
                    .child(
                        text(
                            "A modern desktop environment with GPU-accelerated shader effects",
                            0, 0
                        )
                        .at(60, 140)
                        .size(20.0)
                        .color(Color::TEXT_TERTIARY)
                        .font("regular"),
                    )
                    .child(
                        vstack(0, 0)
                            .at(60, 200)
                            .spacing(12)
                            .child(
                                text("• Native Wayland compositor", 0, 0)
                                    .at(60, 200)
                                    .size(17.0)
                                    .color(Color::TEXT_SECONDARY)
                                    .font("regular"),
                            )
                            .child(
                                text("• Hardware-accelerated GPU rendering", 0, 0)
                                    .at(60, 239)
                                    .size(17.0)
                                    .color(Color::TEXT_SECONDARY)
                                    .font("regular"),
                            )
                            .child(
                                text("• Real-time shader effects (blur, glow, gradients)", 0, 0)
                                    .at(60, 278)
                                    .size(17.0)
                                    .color(Color::TEXT_SECONDARY)
                                    .font("regular"),
                            )
                            .child(
                                text("• Built with Smithay Client Toolkit + glam", 0, 0)
                                    .at(60, 317)
                                    .size(17.0)
                                    .color(Color::TEXT_SECONDARY)
                                    .font("regular"),
                            )
                            .child(
                                text("• Custom GLSL-like fragment shaders", 0, 0)
                                    .at(60, 356)
                                    .size(17.0)
                                    .color(Color::TEXT_SECONDARY)
                                    .font("regular"),
                            ),
                    ),
            )
            .child(
                // Section title
                text("Shader Effects Gallery", 0, 0)
                    .at(margin, 460)
                    .size(24.0)
                    .color(Color::TEXT_PRIMARY)
                    .font("bold"),
            )
            .child(
                // Demo card 1: Gradient effect
                card(0, 0, 0, 0)
                    .frame(margin, 500, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(100, 50, 200))
                    .gradient(Color::rgb(200, 50, 150), 45.0)
                    .child(
                        text("Gradient", 0, 0)
                            .at(margin + 15, 520)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("Linear gradient", 0, 0)
                            .at(margin + 15, 545)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Demo card 2: Box Blur - DISABLED (too slow for CPU)
                card(0, 0, 0, 0)
                    .frame(margin + demo_card_width + 10, 500, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(50, 150, 255))
                    .blur(1.5)
                    .child(
                        text("Box Blur", 0, 0)
                            .at(margin + demo_card_width + 25, 520)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("Fast blur", 0, 0)
                            .at(margin + demo_card_width + 25, 545)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Demo card 3: Glow effect
                card(0, 0, 0, 0)
                    .frame(margin + (demo_card_width + 10) * 2, 500, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(255, 150, 50))
                    .glow(1.2)
                    .child(
                        text("Glow", 0, 0)
                            .at(margin + (demo_card_width + 10) * 2 + 15, 520)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("Luminance glow", 0, 0)
                            .at(margin + (demo_card_width + 10) * 2 + 15, 545)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Demo card 4: Drop Shadow - DISABLED (too slow for CPU)
                card(0, 0, 0, 0)
                    .frame(margin + (demo_card_width + 10) * 3, 500, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(100, 200, 100))
                    .shadow(true)
                    .shadow_blur(10)
                    .child(
                        text("Drop Shadow", 0, 0)
                            .at(margin + (demo_card_width + 10) * 3 + 15, 520)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("Soft shadow", 0, 0)
                            .at(margin + (demo_card_width + 10) * 3 + 15, 545)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Row 2 - Demo card 5: Contrast
                card(0, 0, 0, 0)
                    .frame(margin, 660, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(150, 150, 150))
                    .contrast(2.0)
                    .child(
                        text("Contrast", 0, 0)
                            .at(margin + 15, 680)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("High contrast", 0, 0)
                            .at(margin + 15, 705)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Demo card 6: Gaussian Blur (custom shader)
                // Demo card 6: Gaussian Blur - DISABLED (too slow for CPU)
                card(0, 0, 0, 0)
                    .frame(margin + demo_card_width + 10, 660, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(255, 100, 150))
                    .blur(1.5)
                    .child(
                        text("Gaussian", 0, 0)
                            .at(margin + demo_card_width + 25, 680)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("Quality blur", 0, 0)
                            .at(margin + demo_card_width + 25, 705)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Demo card 7: Vertical gradient
                card(0, 0, 0, 0)
                    .frame(margin + (demo_card_width + 10) * 2, 660, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(50, 200, 150))
                    .gradient(Color::rgb(50, 100, 200), 90.0)
                    .child(
                        text("Vertical", 0, 0)
                            .at(margin + (demo_card_width + 10) * 2 + 15, 680)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("90° gradient", 0, 0)
                            .at(margin + (demo_card_width + 10) * 2 + 15, 705)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            )
            .child(
                // Demo card 8: Brightness
                card(0, 0, 0, 0)
                    .frame(margin + (demo_card_width + 10) * 3, 660, demo_card_width, demo_card_height)
                    .rounded(12.0)
                    .background(Color::rgb(200, 100, 50))
                    .brightness(1.4)
                    .child(
                        text("Brightness", 0, 0)
                            .at(margin + (demo_card_width + 10) * 3 + 15, 680)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("semibold"),
                    )
                    .child(
                        text("Enhanced", 0, 0)
                            .at(margin + (demo_card_width + 10) * 3 + 15, 705)
                            .size(12.0)
                            .color(Color::rgba(255, 255, 255, 200))
                            .font("regular"),
                    ),
            );

        // Render the UI tree
        ui.render(canvas, &text_renderer);
    });

    window.run()
}
