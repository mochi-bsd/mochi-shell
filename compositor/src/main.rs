use std::sync::Arc;
use slog::{Drain, Logger, o};

use smithay::{
    reexports::wayland_server::Display,
    backend::{
        winit::{self, WinitEvent},
        renderer::gles::GlesRenderer,
    },
};

use wayland_server::ListeningSocket;

mod state;
use state::{HanamiState, ClientState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = Logger::root(drain, o!());

    slog::info!(log, "╔══════════════════════════════════════════╗");
    slog::info!(log, "║   Hanami Compositor v{}          ║", env!("CARGO_PKG_VERSION"));
    slog::info!(log, "║   Wayland Compositor for Mochi Desktop   ║");
    slog::info!(log, "╚══════════════════════════════════════════╝");

    // Check if we should use DRM backend (TTY) or Winit backend (nested)
    let use_drm = std::env::var("HANAMI_USE_DRM").is_ok() || 
                  std::env::var("XDG_SESSION_TYPE").map(|s| s == "tty").unwrap_or(false);

    if use_drm {
        slog::info!(log, "DRM/KMS backend not yet implemented");
        slog::info!(log, "Please run in a Wayland or X11 session for now");
        return Ok(());
    }

    // Use Winit backend (nested compositor)
    slog::info!(log, "Starting with Winit backend (nested mode)");
    run_winit(log)
}

fn run_winit(log: Logger) -> Result<(), Box<dyn std::error::Error>> {
    // Create Wayland display
    let mut display: Display<HanamiState> = Display::new()?;
    let dh = display.handle();

    // Initialize compositor state
    let compositor_state = smithay::wayland::compositor::CompositorState::new::<HanamiState>(&dh);
    let xdg_shell_state = smithay::wayland::shell::xdg::XdgShellState::new::<HanamiState>(&dh);
    let shm_state = smithay::wayland::shm::ShmState::new::<HanamiState>(&dh, vec![]);
    let seat_state = smithay::input::SeatState::new();

    let mut state = HanamiState {
        log: log.clone(),
        compositor_state,
        xdg_shell_state,
        shm_state,
        seat_state,
        windows: Vec::new(),
    };

    // Create Wayland socket - try wayland-0 first, then increment
    let socket_name = std::env::var("WAYLAND_DISPLAY").ok();
    
    let listener = if let Some(name) = socket_name {
        slog::info!(log, "Using WAYLAND_DISPLAY from environment: {}", name);
        ListeningSocket::bind(&name)?
    } else {
        // Try wayland-0, wayland-1, wayland-2, etc. until we find an available socket
        let mut display_num = 0;
        loop {
            let socket_name = format!("wayland-{}", display_num);
            match ListeningSocket::bind(&socket_name) {
                Ok(listener) => {
                    slog::info!(log, "Bound to {} (display #{})", socket_name, display_num);
                    break listener;
                }
                Err(_) if display_num < 32 => {
                    slog::debug!(log, "{} already in use, trying next...", socket_name);
                    display_num += 1;
                }
                Err(e) => {
                    slog::error!(log, "Failed to bind to any wayland socket: {}", e);
                    return Err(e.into());
                }
            }
        }
    };
    
    let socket_name = listener
        .socket_name()
        .and_then(|s| s.to_str())
        .unwrap_or("wayland-0")
        .to_string();
    
    // Print to stdout for scripts to capture (like cage does)
    println!("{}", socket_name);
    
    slog::info!(log, "");
    slog::info!(log, "✓ Wayland socket ready: {}", socket_name);
    slog::info!(log, "✓ Clients can connect with: WAYLAND_DISPLAY={}", socket_name);
    slog::info!(log, "");
    
    // Set WAYLAND_DISPLAY for child processes
    std::env::set_var("WAYLAND_DISPLAY", &socket_name);

    // Initialize Winit backend
    let (mut backend, mut winit_evt_loop) = winit::init::<GlesRenderer>()?;
    
    slog::info!(log, "✓ Display window created");
    slog::info!(log, "✓ Compositor ready - waiting for clients...");

    let mut clients = Vec::new();

    // Main event loop
    loop {
        // Accept new clients
        if let Some(stream) = listener.accept()? {
            slog::info!(log, "New client connected");
            let client = display
                .handle()
                .insert_client(stream, Arc::new(ClientState::default()))?;
            clients.push(client);
        }

        // Dispatch Winit events
        let mut should_exit = false;
        winit_evt_loop.dispatch_new_events(|event| {
            match event {
                WinitEvent::CloseRequested => {
                    slog::info!(log, "Close requested, shutting down");
                    should_exit = true;
                }
                WinitEvent::Resized { size, .. } => {
                    slog::debug!(log, "Window resized"; "width" => size.w, "height" => size.h);
                }
                WinitEvent::Input(input_event) => {
                    slog::trace!(log, "Input event"; "event" => ?input_event);
                }
                _ => {}
            }
        });
        
        if should_exit {
            break;
        }

        // Dispatch client requests
        display.dispatch_clients(&mut state)?;
        display.flush_clients()?;

        // Render
        {
            let (_renderer, _framebuffer) = backend.bind()?;
            // TODO: Render windows here
            // For now, just clear to a dark background (done automatically)
        }
        backend.submit(None)?;

        // Small sleep to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }
    
    Ok(())
}
