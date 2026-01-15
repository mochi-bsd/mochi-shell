use std::sync::Arc;
use slog::{Drain, Logger, o};

use smithay::{
    delegate_compositor, delegate_shm, delegate_xdg_shell,
    reexports::wayland_server::Display,
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shell::xdg::{XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState},
        shm::{ShmHandler, ShmState},
        buffer::BufferHandler,
    },
};

use wayland_server::{
    backend::{ClientData, ClientId, DisconnectReason},
    protocol::wl_surface::WlSurface,
    Client, ListeningSocket,
};

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

    // Create Wayland display
    let mut display: Display<HanamiState> = Display::new()?;
    let dh = display.handle();

    // Initialize compositor state
    let compositor_state = CompositorState::new::<HanamiState>(&dh);
    let xdg_shell_state = XdgShellState::new::<HanamiState>(&dh);
    let shm_state = ShmState::new::<HanamiState>(&dh, vec![]);
    let seat_state = smithay::input::SeatState::new();

    let mut state = HanamiState {
        log: log.clone(),
        compositor_state,
        xdg_shell_state,
        shm_state,
        seat_state,
        windows: Vec::new(),
    };

    // Create Wayland socket
    let socket_name = std::env::var("WAYLAND_DISPLAY")
        .unwrap_or_else(|_| "wayland-hanami".to_string());
    
    let listener = ListeningSocket::bind(&socket_name)?;
    slog::info!(log, "Listening on socket: {}", socket_name);
    slog::info!(log, "Set WAYLAND_DISPLAY={} to connect clients", socket_name);

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

        // Dispatch client requests
        display.dispatch_clients(&mut state)?;
        display.flush_clients()?;

        // Small sleep to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
