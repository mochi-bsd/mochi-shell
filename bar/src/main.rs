use anyhow::Result;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{
    App, Application, AssetSource, Bounds, Context, SharedString, Size, TextRenderingMode, Window,
    WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowOptions, div, point,
    prelude::*, px, rgb, rgba, svg, white,
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
use gpui::{layer_shell::*, WindowKind};

// Embed custom fonts
const INTER_REGULAR: &[u8] =
    include_bytes!("../../fs/library/shared/fonts/Inter-Regular.ttf");
const INTER_LIGHT: &[u8] = include_bytes!("../../fs/library/shared/fonts/Inter-Light.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../fs/library/shared/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] =
    include_bytes!("../../fs/library/shared/fonts/Inter-SemiBold.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../fs/library/shared/fonts/Inter-Bold.ttf");

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

// Global state for active window tracking
#[derive(Clone)]
struct ActiveWindowState {
    app_name: Arc<Mutex<String>>,
    window_title: Arc<Mutex<String>>,
}

impl ActiveWindowState {
    fn new() -> Self {
        Self {
            app_name: Arc::new(Mutex::new("Shell Explorer".to_string())),
            window_title: Arc::new(Mutex::new(String::new())),
        }
    }

    fn get_app_name(&self) -> String {
        self.app_name.lock().unwrap().clone()
    }

    fn set_active_window(&self, app_name: String, title: String) {
        *self.app_name.lock().unwrap() = app_name;
        *self.window_title.lock().unwrap() = title;
    }
}

struct StatusBar {
    active_window_state: ActiveWindowState,
}

impl StatusBar {
    fn new(cx: &mut Context<Self>, active_window_state: ActiveWindowState) -> Self {
        // Update the time every second
        cx.spawn(async move |this, cx| {
            loop {
                let _ = this.update(cx, |_, cx| cx.notify());
                cx.background_executor().timer(Duration::from_secs(1)).await;
            }
        })
        .detach();

        // Start window tracking in background
        let state = active_window_state.clone();
        cx.spawn(async move |this, cx| {
            loop {
                // Try to get active window info from different sources
                if let Ok(info) = get_active_window_info().await {
                    state.set_active_window(info.0, info.1);
                    let _ = this.update(cx, |_, cx| cx.notify());
                }
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
            }
        })
        .detach();

        StatusBar {
            active_window_state,
        }
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let now = chrono::Local::now();
        let time_str = now.format("%a %-d %B %H:%M").to_string();
        let active_app = self.active_window_state.get_app_name();

        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(40.0))
            .px_4()
            .font_family("Inter")
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
                            .w(px(24.0))
                            .h(px(24.0))
                            .cursor_pointer()
                            .rounded_sm()
                            .hover(|style| style.bg(rgba(0x2a2a2a80)))
                            .child(
                                svg()
                                    .size(px(14.0))
                                    .path("assets/logo.svg")
                                    .text_color(white()),
                            ),
                    )
                    .child(
                        // Workspace name
                        div()
                            .flex()
                            .items_center()
                            .px_2()
                            .h(px(24.0))
                            .text_color(white())
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Workspace 1"),
                    )
                    .child(
                        // Separator
                        div().w(px(1.0)).h(px(14.0)).bg(rgba(0x40404080)),
                    )
                    .child(
                        // Menu items with active app name
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(menu_item_active(active_app))
                            .child(menu_item("File".to_string()))
                            .child(menu_item("Edit".to_string()))
                            .child(menu_item("View".to_string()))
                            .child(menu_item("Options".to_string()))
                            .child(menu_item("Action".to_string()))
                            .child(menu_item("Help".to_string()))
                            .child(menu_item("Window".to_string())),
                    ),
            )
            .child(
                // Right side - time with date
                div()
                    .flex()
                    .items_center()
                    .text_color(white())
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(time_str),
            )
    }
}

fn menu_item(label: String) -> impl IntoElement {
    div()
        .px_2()
        .h(px(22.0))
        .flex()
        .items_center()
        .text_color(rgba(0xccccccFF))
        .text_sm()
        .cursor_pointer()
        .rounded_sm()
        .hover(|style| style.bg(rgba(0x2a2a2a80)).text_color(white()))
        .child(label)
}

fn menu_item_active(label: String) -> impl IntoElement {
    div()
        .px_2()
        .h(px(22.0))
        .flex()
        .items_center()
        .text_color(white())
        .text_sm()
        .font_weight(gpui::FontWeight(700.0))
        .cursor_pointer()
        .rounded_sm()
        .hover(|style| style.bg(rgba(0x2a2a2a80)))
        .child(label)
}

// Get active window info from various sources
async fn get_active_window_info() -> Result<(String, String)> {
    // Try xdotool + xprop (works with XWayland and most apps on GNOME Wayland)
    // First get the window ID
    let window_id_output = std::process::Command::new("xdotool")
        .args(["getactivewindow"])
        .output();
    
    if let Ok(window_id_output) = window_id_output {
        if window_id_output.status.success() {
            let window_id = String::from_utf8_lossy(&window_id_output.stdout).trim().to_string();
            
            if !window_id.is_empty() {
                // Get WM_CLASS using xprop (this gives us the real app name)
                let class_output = std::process::Command::new("xprop")
                    .args(["-id", &window_id, "WM_CLASS"])
                    .output();
                
                if let Ok(class_output) = class_output {
                    if class_output.status.success() {
                        let class_str = String::from_utf8_lossy(&class_output.stdout);
                        // WM_CLASS format: WM_CLASS(STRING) = "instance", "Class"
                        // We want the second part (Class)
                        if let Some(app_name) = extract_wm_class(&class_str) {
                            // Also get the window title for additional context
                            let title_output = std::process::Command::new("xdotool")
                                .args(["getactivewindow", "getwindowname"])
                                .output();
                            
                            let title = if let Ok(title_output) = title_output {
                                String::from_utf8_lossy(&title_output.stdout).trim().to_string()
                            } else {
                                String::new()
                            };
                            
                            return Ok((app_name, title));
                        }
                    }
                }
            }
        }
    }

    // Fallback - suggest installing xdotool and xprop
    static LOGGED_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED_ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!("Window tracking not available. Install xdotool and x11-utils:");
        eprintln!("  sudo apt install xdotool x11-utils");
    }
    
    Ok(("Shell Explorer".to_string(), String::new()))
}

fn extract_wm_class(output: &str) -> Option<String> {
    // Parse WM_CLASS output: WM_CLASS(STRING) = "instance", "Class"
    // We want the second quoted string (Class)
    let parts: Vec<&str> = output.split('"').collect();
    if parts.len() >= 4 {
        // parts[0] = "WM_CLASS(STRING) = "
        // parts[1] = "instance"
        // parts[2] = ", "
        // parts[3] = "Class"
        Some(parts[3].to_string())
    } else {
        None
    }
}

fn main() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        })
        .run(|cx: &mut App| {
            // Enable subpixel font rendering for better text quality
            cx.set_text_rendering_mode(TextRenderingMode::Subpixel);

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

            // Set up theme with Inter font
            // Note: Font smoothing is enabled by default on Linux (subpixel antialiasing)
            // Optical sizing (opsz) would require font feature support in GPUI's text system
            let mut theme = Theme::default();
            theme.font_family = "Inter".into(); // Use Inter Display for better rendering
            theme.font_size = px(14.);
            cx.set_global(theme);

            let active_window_state = ActiveWindowState::new();

            // Try layer shell first (for compositors that support it: GNOME, Sway, Hyprland, Cage)
            #[cfg(all(any(target_os = "linux", target_os = "freebsd"), feature = "wayland"))]
            let result = {
                let state = active_window_state.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: None, // Let layer shell handle positioning
                        app_id: Some("mochi-bar".to_string()),
                        window_background: WindowBackgroundAppearance::Transparent,
                        window_decorations: Some(WindowDecorations::Client),
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "panel".to_string(),
                            anchor: Anchor::LEFT | Anchor::RIGHT | Anchor::TOP,
                            margin: Some((px(0.), px(0.), px(0.), px(0.))), // No margins for HiDPI
                            keyboard_interactivity: KeyboardInteractivity::None,
                            layer: Layer::Top,
                            exclusive_zone: Some(32), // Match bar height
                            ..Default::default()
                        }),
                        focus: false,
                        show: true,
                        is_movable: false,
                        is_resizable: false,
                        is_minimizable: false,
                        ..Default::default()
                    },
                    move |_, cx| cx.new(|cx| StatusBar::new(cx, state)),
                )
            };

            // Fallback to regular window if layer shell fails
            #[cfg(all(any(target_os = "linux", target_os = "freebsd"), feature = "wayland"))]
            if result.is_err() {
                eprintln!("Layer shell not supported, falling back to regular window");
                let state = active_window_state.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Maximized(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: Size::new(px(1920.), px(32.)),
                        })),
                        app_id: Some("mochi-bar".to_string()),
                        window_background: WindowBackgroundAppearance::Transparent,
                        window_decorations: Some(WindowDecorations::Client),
                        focus: false,
                        show: true,
                        is_movable: false,
                        is_resizable: false,
                        is_minimizable: false,
                        ..Default::default()
                    },
                    move |_, cx| cx.new(|cx| StatusBar::new(cx, state)),
                )
                .unwrap();
            }

            // Non-wayland platforms use regular window
            #[cfg(not(all(
                any(target_os = "linux", target_os = "freebsd"),
                feature = "wayland"
            )))]
            {
                let state = active_window_state.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Maximized(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: Size::new(px(1920.), px(32.)),
                        })),
                        app_id: Some("mochi-bar".to_string()),
                        window_background: WindowBackgroundAppearance::Transparent,
                        window_decorations: Some(WindowDecorations::Client),
                        focus: false,
                        show: true,
                        is_movable: false,
                        is_resizable: false,
                        is_minimizable: false,
                        ..Default::default()
                    },
                    move |_, cx| cx.new(|cx| StatusBar::new(cx, state)),
                )
                .unwrap();
            }
        });
}
