use mochi::{Canvas, Color, TextRenderer, Window, WindowConfig, Element};
use mochi::{container, card, text, vstack, titlebar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load fonts
    let mut text_renderer = TextRenderer::new();
    let inter_regular = include_bytes!("../../fs/library/shared/fonts/Inter-Regular.ttf");
    text_renderer.load_font("regular", inter_regular)?;
    let inter_semibold = include_bytes!("../../fs/library/shared/fonts/Inter-SemiBold.ttf");
    text_renderer.load_font("semibold", inter_semibold)?;

    // Create window
    let config = WindowConfig {
        title: "Mochi Desktop Environment".to_string(),
        width: 800,
        height: 600,
        min_width: Some(600),
        min_height: Some(400),
        decorations: false, // Will auto-detect and use server decorations on GNOME/KDE
        shadow: true,
        border: true,
    };

    let mut window = Window::new(config)?;

    // Build UI using DSL
    let ui = container(0, 0, 800, 600)
        .background(Color::BG_PRIMARY)
        .child(
            // Custom titlebar
            titlebar(800, "Mochi Desktop")
        )
        .child(
            // Main card
            card(50, 52, 700, 488)
                .shadow(false)
                .child(
                    text("Welcome to Mochi", 80, 102)
                        .size(36.0)
                        .color(Color::TEXT_PRIMARY)
                        .font("semibold")
                )
                .child(
                    text("A modern desktop environment built with Rust and Wayland", 80, 152)
                        .size(18.0)
                        .color(Color::TEXT_TERTIARY)
                        .font("regular")
                )
                .child(
                    vstack(80, 212)
                        .spacing(10)
                        .child(
                            text("• Native Wayland compositor", 80, 212)
                                .size(16.0)
                                .color(Color::TEXT_SECONDARY)
                                .font("regular")
                        )
                        .child(
                            text("• Hardware-accelerated rendering", 80, 247)
                                .size(16.0)
                                .color(Color::TEXT_SECONDARY)
                                .font("regular")
                        )
                        .child(
                            text("• Modern UI with dark mode", 80, 282)
                                .size(16.0)
                                .color(Color::TEXT_SECONDARY)
                                .font("regular")
                        )
                        .child(
                            text("• Built with Smithay Client Toolkit", 80, 317)
                                .size(16.0)
                                .color(Color::TEXT_SECONDARY)
                                .font("regular")
                        )
                )
        );

    // Set up draw callback
    window.on_draw(move |canvas: &mut Canvas| {
        ui.render(canvas, &text_renderer);
    });

    window.run()
}
