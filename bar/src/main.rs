use mochi::{div, text, Canvas, Color, Element, TextRenderer, Window, WindowConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

// Get active window info from various sources
fn get_active_window_info() -> Result<(String, String), Box<dyn std::error::Error>> {
    // Try xdotool + xprop (works with XWayland and most apps on GNOME Wayland)
    let window_id_output = std::process::Command::new("xdotool")
        .args(["getactivewindow"])
        .output();
    
    if let Ok(window_id_output) = window_id_output {
        if window_id_output.status.success() {
            let window_id = String::from_utf8_lossy(&window_id_output.stdout).trim().to_string();
            
            if !window_id.is_empty() {
                // Get WM_CLASS using xprop
                let class_output = std::process::Command::new("xprop")
                    .args(["-id", &window_id, "WM_CLASS"])
                    .output();
                
                if let Ok(class_output) = class_output {
                    if class_output.status.success() {
                        let class_str = String::from_utf8_lossy(&class_output.stdout);
                        if let Some(app_name) = extract_wm_class(&class_str) {
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
    
    Ok(("Shell Explorer".to_string(), String::new()))
}

fn extract_wm_class(output: &str) -> Option<String> {
    // Parse WM_CLASS output: WM_CLASS(STRING) = "instance", "Class"
    let parts: Vec<&str> = output.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load fonts
    let mut text_renderer = TextRenderer::new();
    let inter_regular = include_bytes!("../../fs/library/shared/fonts/Inter-Regular.ttf");
    text_renderer.load_font("regular", inter_regular)?;
    let inter_medium = include_bytes!("../../fs/library/shared/fonts/Inter-Medium.ttf");
    text_renderer.load_font("medium", inter_medium)?;
    let inter_semibold = include_bytes!("../../fs/library/shared/fonts/Inter-SemiBold.ttf");
    text_renderer.load_font("semibold", inter_semibold)?;
    let inter_bold = include_bytes!("../../fs/library/shared/fonts/Inter-Bold.ttf");
    text_renderer.load_font("bold", inter_bold)?;
    
    // Create window for status bar
    let config = WindowConfig {
        title: "Mochi Bar".to_string(),
        width: 1920,
        height: 32,
        min_width: None,
        min_height: None,
        decorations: false,
        transparent: false, // Test with non-transparent first
        draggable: false,
    };

    let mut window = Window::new(config)?;
    
    // Active window state
    let active_window_state = ActiveWindowState::new();
    let state_clone = active_window_state.clone();
    
    // Spawn background thread to update active window info
    std::thread::spawn(move || {
        loop {
            if let Ok(info) = get_active_window_info() {
                state_clone.set_active_window(info.0, info.1);
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    // Set up draw callback
    window.on_draw(move |canvas: &mut Canvas| {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        
        // Get current time
        let now = chrono::Local::now();
        let time_str = now.format("%a %-d %B %H:%M").to_string();
        
        // Get active app name
        let active_app = active_window_state.get_app_name();

        // Build UI tree - test with background first to see shadows
        let ui = div(0, 0, width, height)
            .background(Color::rgba(255,255,255, 255))
            .child(
                text("◆", 0, 0)
                    .at(8, 8)
                    .size(14.0)
                    .color(Color::rgba(0, 0, 0, 255))
                    .font("bold")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 80)),
            )
            .child(
                text("Workspace 1", 0, 0)
                    .at(32, 8)
                    .size(13.0)
                    .color(Color::rgba(0, 0, 0, 255))
                    .font("semibold")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 80)),
            )
            .child(
                text(&active_app, 0, 0)
                    .at(140, 8)
                    .size(13.0)
                    .color(Color::rgba(0, 0, 0, 255))
                    .font("bold")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 80)),
            )
            .child(
                text("File", 0, 0)
                    .at(140 + (active_app.len() as i32 * 8) + 20, 8)
                    .size(13.0)
                    .color(Color::rgba(40, 40, 40, 255))
                    .font("regular")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 70)),
            )
            .child(
                text("Edit", 0, 0)
                    .at(140 + (active_app.len() as i32 * 8) + 60, 8)
                    .size(13.0)
                    .color(Color::rgba(40, 40, 40, 255))
                    .font("regular")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 70)),
            )
            .child(
                text("View", 0, 0)
                    .at(140 + (active_app.len() as i32 * 8) + 100, 8)
                    .size(13.0)
                    .color(Color::rgba(40, 40, 40, 255))
                    .font("regular")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 70)),
            )
            .child(
                text("Options", 0, 0)
                    .at(140 + (active_app.len() as i32 * 8) + 150, 8)
                    .size(13.0)
                    .color(Color::rgba(40, 40, 40, 255))
                    .font("regular")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 70)),
            )
            .child(
                text(&time_str, 0, 0)
                    .at(width - 120, 8)
                    .size(13.0)
                    .color(Color::rgba(0, 0, 0, 255))
                    .font("medium")
                    .shadow(true)
                    .shadow_offset(1, 1)
                    .shadow_blur(2)
                    .shadow_color(Color::rgba(0, 0, 0, 80)),
            );

        // Render the UI tree
        ui.render(canvas, &text_renderer);
    });

    // Run the window event loop
    window.run()
}
