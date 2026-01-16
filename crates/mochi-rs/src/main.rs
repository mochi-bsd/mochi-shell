use mochi::{container, div, text, titlebar, vstack};
use mochi::{Canvas, Color, Element, TextRenderer, Window, WindowConfig};

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
        title: "Mochi Desktop - Software Renderer".to_string(),
        width: 1400,
        height: 900,
        min_width: Some(800),
        min_height: Some(600),
        decorations: false,
    };

    let mut window = Window::new(config)?;

    // Set up draw callback with RSX-like declarative UI
    window.on_draw(move |canvas: &mut Canvas| {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;

        // Layout calculations
        let margin = 40;
        let card_width = width - (margin * 2);
        let card_height = 380;
        let demo_div_width = (card_width - 30) / 4;
        let demo_div_height = 140;

        // Build UI tree with RSX-like style
        let ui = container(0, 0, 0, 0)
            .frame(0, 0, width, height)
            .background(Color::BG_PRIMARY)
            .child(
                // Titlebar with gradient effect
                titlebar(width, "Mochi Desktop - LLVMpipe Software Renderer")
                    .background(Color::rgb(40, 40, 50)),
            )
            .child(
                // Main content div with shadow (only one shadow for performance)
                div(0, 0, 0, 0)
                    .frame(margin, 60, card_width, card_height)
                    .rounded(16.0)
                    .child(
                        text("Welcome to Mochi", 0, 0)
                            .at(60, 80)
                            .size(42.0)
                            .color(Color::TEXT_PRIMARY)
                            .font("semibold"),
                    )
                    .child(
                        text(
                            "A modern desktop environment with software-accelerated rendering",
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
                                text("• LLVMpipe software rendering", 0, 0)
                                    .at(60, 239)
                                    .size(17.0)
                                    .color(Color::TEXT_SECONDARY)
                                    .font("regular"),
                            )
                            .child(
                                text("• Real-time effects (blur, shadows, gradients)", 0, 0)
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
                            ),
                    ),
            );
        // Render the UI tree
        ui.render(canvas, &text_renderer);
    });

    // Run the window event loop
    window.run()
}
