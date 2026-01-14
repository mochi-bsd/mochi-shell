use anyhow::Result;
use std::path::PathBuf;

use gpui::{
    App, Application, AssetSource, Bounds, Context, SharedString, Size, Window,
    WindowBackgroundAppearance, WindowBounds, WindowOptions, div, img, point, prelude::*, px,
};
use gpui_component::theme::Theme;

#[cfg(all(
    any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ),
    feature = "wayland"
))]
use gpui::{WindowKind, layer_shell::*};

// Embed custom fonts
const INTER_REGULAR: &[u8] =
    include_bytes!("../../fs/library/shared/fonts/InterDisplay-Regular.ttf");
const INTER_LIGHT: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-Light.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-Medium.ttf");
const INTER_SEMIBOLD: &[u8] =
    include_bytes!("../../fs/library/shared/fonts/InterDisplay-SemiBold.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-Bold.ttf");

struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        std::fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        std::fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

struct Shell {
    background_path: PathBuf,
}

impl Shell {
    fn new(cx: &mut Context<Self>) -> Self {
        println!("=== Mochi Desktop Shell Initializing ===");

        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        println!("HOME directory: {}", home);

        let background_path = PathBuf::from(&home).join("Photos/Default.jpg");
        println!("Looking for background at: {:?}", background_path);
        println!("Background path exists: {}", background_path.exists());

        if background_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&background_path) {
                println!("Background file size: {} bytes", metadata.len());
            }
        }

        println!("=== Shell Initialization Complete ===\n");
        println!("NOTE: Launch bar separately with: /opt/mochidesktop/bar");

        Shell { background_path }
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        println!(
            "DEBUG: Render called - background_path: {:?}",
            self.background_path
        );

        div()
            .flex()
            .relative()
            .w_full()
            .h_full()
            .font_family("Inter Display")
            .child(
                // Background image - absolute positioned to fill entire screen
                img(self.background_path.clone())
                    .w_full()
                    .h_full()
                    .object_fit(gpui::ObjectFit::Cover)
                    .absolute()
                    .top_0()
                    .left_0(),
            )
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        println!("Shell shutting down");
    }
}

fn main() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        })
        .run(|cx: &mut App| {
            // Register custom fonts
            cx.text_system()
                .add_fonts(vec![
                    std::borrow::Cow::Borrowed(INTER_REGULAR),
                    std::borrow::Cow::Borrowed(INTER_LIGHT),
                    std::borrow::Cow::Borrowed(INTER_MEDIUM),
                    std::borrow::Cow::Borrowed(INTER_BOLD),
                    std::borrow::Cow::Borrowed(INTER_SEMIBOLD),
                ])
                .expect("Failed to load fonts");

            // Set up theme with Inter Variable font
            let mut theme = Theme::default();
            theme.font_family = "Inter Variable".into();
            theme.font_size = px(14.);
            cx.set_global(theme);

            // Create fullscreen shell window
            #[cfg(all(
                any(
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "openbsd",
                    target_os = "netbsd"
                ),
                feature = "wayland"
            ))]
            let result = cx.open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Fullscreen(Bounds {
                        origin: point(px(0.), px(0.)),
                        size: Size::new(px(1920.), px(1080.)),
                    })),
                    app_id: Some("mochi-shell".to_string()),
                    window_background: WindowBackgroundAppearance::Opaque,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "shell".to_string(),
                        anchor: Anchor::LEFT | Anchor::RIGHT | Anchor::TOP | Anchor::BOTTOM,
                        margin: Some((px(0.), px(0.), px(0.), px(0.))),
                        keyboard_interactivity: KeyboardInteractivity::None,
                        layer: Layer::Background,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(Shell::new),
            );

            // Fallback to regular fullscreen window
            #[cfg(all(
                any(
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "openbsd",
                    target_os = "netbsd"
                ),
                feature = "wayland"
            ))]
            if result.is_err() {
                eprintln!("Layer shell not supported, falling back to fullscreen window");
                cx.open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Fullscreen(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: Size::new(px(1920.), px(1080.)),
                        })),
                        app_id: Some("mochi-shell".to_string()),
                        window_background: WindowBackgroundAppearance::Opaque,
                        ..Default::default()
                    },
                    |_, cx| cx.new(Shell::new),
                )
                .unwrap();
            }

            // Non-wayland platforms
            #[cfg(not(all(
                any(
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "openbsd",
                    target_os = "netbsd"
                ),
                feature = "wayland"
            )))]
            cx.open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Fullscreen(Bounds {
                        origin: point(px(0.), px(0.)),
                        size: Size::new(px(1920.), px(1080.)),
                    })),
                    app_id: Some("mochi-shell".to_string()),
                    window_background: WindowBackgroundAppearance::Opaque,
                    ..Default::default()
                },
                |_, cx| cx.new(Shell::new),
            )
            .unwrap();
        });
}
