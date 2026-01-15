use slog::Logger;
use smithay::{
    delegate_compositor, delegate_shm, delegate_xdg_shell, delegate_seat,
    input::{Seat, SeatHandler, SeatState},
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shell::xdg::{
            XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, 
            PositionerState,
        },
        shm::{ShmHandler, ShmState},
        buffer::BufferHandler,
    },
};
use wayland_server::{
    backend::{ClientData, ClientId, DisconnectReason},
    protocol::{wl_surface::WlSurface, wl_seat::WlSeat},
    Client,
};

pub struct HanamiState {
    pub log: Logger,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub windows: Vec<ToplevelSurface>,
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("Client initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("Client disconnected");
    }
}

// Compositor handler implementation
impl CompositorHandler for HanamiState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        slog::debug!(self.log, "Surface committed"; "surface" => ?surface);
    }
}

// XDG Shell handler implementation
impl XdgShellHandler for HanamiState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        slog::info!(self.log, "New toplevel window created");
        self.windows.push(surface);
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        slog::info!(self.log, "New popup created");
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: smithay::utils::Serial) {
        // Handle popup grab
    }

    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {
        // Handle reposition request
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        slog::info!(self.log, "Toplevel window destroyed");
        self.windows.retain(|w| w != &surface);
    }

    fn popup_destroyed(&mut self, _surface: PopupSurface) {
        slog::info!(self.log, "Popup destroyed");
    }
}

// Seat handler implementation
impl SeatHandler for HanamiState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {
        // Handle focus changes
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {
        // Handle cursor image changes
    }
}

// SHM handler implementation
impl ShmHandler for HanamiState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

// Buffer handler implementation
impl BufferHandler for HanamiState {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {
        // Handle buffer destruction
    }
}

// Required trait implementations for state
impl AsMut<CompositorState> for HanamiState {
    fn as_mut(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
}

impl AsMut<XdgShellState> for HanamiState {
    fn as_mut(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
}

impl AsMut<ShmState> for HanamiState {
    fn as_mut(&mut self) -> &mut ShmState {
        &mut self.shm_state
    }
}

// Delegate macros
delegate_compositor!(HanamiState);
delegate_xdg_shell!(HanamiState);
delegate_shm!(HanamiState);
delegate_seat!(HanamiState);
