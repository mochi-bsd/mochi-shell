use std::time::Duration;
use std::path::PathBuf;
use anyhow::Result;

use gpui::{
    App, Application, AssetSource, Bounds, Context, SharedString, Size, Window, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, div, point, prelude::*, px, rgb, svg, white,
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
const INTER_REGULAR: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-Regular.ttf");
const INTER_LIGHT: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-Light.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../fs/library/shared/fonts/InterDisplay-SemiBold.ttf");
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

struct StatusBar {
    active_app_name: SharedString,
}

impl StatusBar {
    fn new(cx: &mut Context<Self>) -> Self {
        // Update the time every second
        cx.spawn(async move |this, cx| {
            loop {
                let _ = this.update(cx, |_, cx| cx.notify());
                cx.background_executor()
                    .timer(Duration::from_secs(1))
                    .await;
            }
        })
        .detach();

        StatusBar {
            active_app_name: "Shell Explorer".into(),
        }
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let now = chrono::Local::now();
        let time_str = now.format("%H:%M (%Y-%m-%d)").to_string();

        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(30.0))
            .bg(rgb(0x1a1a1a))
            .px_2()
            .font_family("Inter Display")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        // Logo button
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(28.0))
                            .h(px(28.0))
                            .cursor_pointer()
                            .rounded_sm()
                            .hover(|style| style.bg(rgb(0x2a2a2a)))
                            .child(
                                svg()
                                    .size(px(16.0))
                                    .path("assets/logo.svg")
                                    .text_color(white())
                            )
                    )
                    .child(
                        // Workspace name
                        div()
                            .flex()
                            .items_center()
                            .px_2()
                            .h(px(28.0))
                            .text_color(white())
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Workspace 1")
                    )
                    .child(
                        // Separator
                        div()
                            .w(px(1.0))
                            .h(px(16.0))
                            .bg(rgb(0x404040))
                    )
                    .child(
                        // Menu items - Cage compositor support
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(menu_item_active("Shell Explorer"))
                            .child(menu_item("File"))
                            .child(menu_item("Edit"))
                            .child(menu_item("View"))
                            .child(menu_item("Options"))
                            .child(menu_item("Action"))
                            .child(menu_item("Help"))
                            .child(menu_item("Window"))
                    )
            )
            .child(
                // Right side - time with date
                div()
                    .flex()
                    .items_center()
                    .text_color(rgb(0xcccccc))
                    .text_sm()
                    .child(time_str)
            )
    }
}

fn menu_item(label: &'static str) -> impl IntoElement {
    div()
        .px_3()
        .h(px(24.0))
        .flex()
        .items_center()
        .text_color(rgb(0xcccccc))
        .text_sm()
        .cursor_pointer()
        .rounded_sm()
        .hover(|style| style.bg(rgb(0x2a2a2a)).text_color(white()))
        .child(label)
}

fn menu_item_active(label: &'static str) -> impl IntoElement {
    div()
        .px_3()
        .h(px(24.0))
        .flex()
        .items_center()
        .text_color(white())
        .text_sm()
        .font_weight(gpui::FontWeight(700.0))
        .cursor_pointer()
        .rounded_sm()
        .hover(|style| style.bg(rgb(0x2a2a2a)))
        .child(label)
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
        // Try layer shell first (for compositors that support it)
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
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(0.), px(30.)),  // Width 0 = let compositor decide
                })),
                app_id: Some("gpui-statusbar".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "statusbar".to_string(),
                    anchor: Anchor::LEFT | Anchor::RIGHT | Anchor::TOP,
                    margin: Some((px(0.), px(0.), px(0.), px(0.))),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    layer: Layer::Top,
                    exclusive_zone: Some(30),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(StatusBar::new),
        );

        // Fallback to regular window if layer shell fails
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
            eprintln!("Layer shell not supported, falling back to regular window");
            cx.open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(0.), px(0.)),
                        size: Size::new(px(0.), px(30.)),
                    })),
                    app_id: Some("gpui-statusbar".to_string()),
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |_, cx| cx.new(StatusBar::new),
            )
            .unwrap();
        }

        // Non-wayland platforms use regular window
        #[cfg(not(all(target_os = "linux", feature = "wayland")))]
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(0.), px(30.)),
                })),
                app_id: Some("gpui-statusbar".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            },
            |_, cx| cx.new(StatusBar::new),
        )
        .unwrap();
    });
}
