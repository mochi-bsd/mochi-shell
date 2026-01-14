use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_shm, delegate_xdg_shell,
    delegate_xdg_window, delegate_pointer, delegate_seat,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{SeatHandler, SeatState, pointer::{PointerHandler, PointerEvent, PointerEventKind}},
    shell::{
        xdg::{
            window::{Window as XdgWindow, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface, wl_pointer, wl_seat},
    Connection, QueueHandle, EventQueue,
};
use wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge;

use crate::canvas::Canvas;
use crate::color::Color;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub decorations: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Mochi Window".to_string(),
            width: 800,
            height: 600,
            min_width: Some(400),
            min_height: Some(300),
            decorations: false, // Use client-side decorations
        }
    }
}

struct AppState {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    xdg_shell_state: XdgShell,
    seat_state: SeatState,
    pool: Option<SlotPool>,
    window: Option<XdgWindow>,
    width: u32,
    height: u32,
    draw_fn: Option<Box<dyn FnMut(&mut Canvas)>>,
    pointer_location: Option<(f64, f64)>,
}

pub struct Window {
    event_queue: EventQueue<AppState>,
    state: AppState,
}

impl Window {
    pub fn new(config: WindowConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::connect_to_env()?;
        let (globals, event_queue) = registry_queue_init::<AppState>(&conn)?;
        let qh = event_queue.handle();

        let state = AppState {
            registry_state: RegistryState::new(&globals),
            output_state: OutputState::new(&globals, &qh),
            compositor_state: CompositorState::bind(&globals, &qh)?,
            shm_state: Shm::bind(&globals, &qh)?,
            xdg_shell_state: XdgShell::bind(&globals, &qh)?,
            seat_state: SeatState::new(&globals, &qh),
            pool: None,
            window: None,
            width: config.width,
            height: config.height,
            draw_fn: None,
            pointer_location: None,
        };

        let mut window = Self {
            event_queue,
            state,
        };

        let qh = window.event_queue.handle();
        let surface = window.state.compositor_state.create_surface(&qh);
        
        // Use client-side decorations for custom titlebar dragging
        let decorations = WindowDecorations::RequestClient;
        
        let xdg_window = window.state.xdg_shell_state.create_window(
            surface,
            decorations,
            &qh,
        );
        
        xdg_window.set_title(&config.title);
        if let (Some(min_w), Some(min_h)) = (config.min_width, config.min_height) {
            xdg_window.set_min_size(Some((min_w, min_h)));
        }
        xdg_window.commit();

        window.state.window = Some(xdg_window);

        Ok(window)
    }

    pub fn on_draw<F>(&mut self, f: F)
    where
        F: FnMut(&mut Canvas) + 'static,
    {
        self.state.draw_fn = Some(Box::new(f));
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.event_queue.blocking_dispatch(&mut self.state)?;
        }
    }
}

impl AppState {
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        let window = match &self.window {
            Some(w) => w,
            None => return,
        };

        let pool = match &mut self.pool {
            Some(p) => p,
            None => return,
        };

        let stride = self.width as i32 * 4;

        let (buffer, canvas_buffer) = pool
            .create_buffer(
                self.width as i32,
                self.height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");

        let mut canvas = Canvas::new(canvas_buffer, self.width, self.height);

        // Clear background
        canvas.clear(Color::BG_PRIMARY);

        // Call user draw function
        if let Some(ref mut draw_fn) = self.draw_fn {
            draw_fn(&mut canvas);
        }

        window.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        window.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);
        window.wl_surface().commit();
    }
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {}

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {}

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {}

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {}

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {}
}

impl WindowHandler for AppState {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &XdgWindow) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        let (width, height) = configure.new_size;
        let mut size_changed = false;
        
        if let Some(w) = width {
            if self.width != w.get() {
                self.width = w.get();
                size_changed = true;
            }
        }
        if let Some(h) = height {
            if self.height != h.get() {
                self.height = h.get();
                size_changed = true;
            }
        }

        // Recreate pool if size changed or pool doesn't exist
        if self.pool.is_none() || size_changed {
            self.pool = Some(
                SlotPool::new(
                    (self.width * self.height * 4) as usize,
                    &self.shm_state
                )
                .expect("Failed to create pool"),
            );
        }

        self.draw(qh);
        window.wl_surface().commit();
    }
}

impl ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

delegate_compositor!(AppState);
delegate_output!(AppState);
delegate_shm!(AppState);
delegate_xdg_shell!(AppState);
delegate_xdg_window!(AppState);
delegate_seat!(AppState);
delegate_pointer!(AppState);
delegate_registry!(AppState);

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        // Request pointer when the capability is available
        if capability == smithay_client_toolkit::seat::Capability::Pointer {
            self.seat_state.get_pointer(qh, &seat).ok();
        }
    }
    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for AppState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        
        for event in events {
            match event.kind {
                Enter { .. } => {
                    self.pointer_location = Some(event.position);
                }
                Leave { .. } => {
                    self.pointer_location = None;
                }
                Motion { .. } => {
                    self.pointer_location = Some(event.position);
                }
                Press { button, serial, .. } => {
                    // Left click
                    if button == 0x110 {
                        // BTN_LEFT
                        if let Some((x, y)) = self.pointer_location {
                            let window_width = self.width as f64;
                            let window_height = self.height as f64;
                            let edge_size = 10.0;
                            let corner_size = 20.0;
                            
                            let at_left = x < edge_size;
                            let at_right = x > window_width - edge_size;
                            let at_top = y < edge_size;
                            let at_bottom = y > window_height - edge_size;
                            
                            let at_left_corner = x < corner_size;
                            let at_right_corner = x > window_width - corner_size;
                            let at_top_corner = y < corner_size;
                            let at_bottom_corner = y > window_height - corner_size;
                            
                            if let Some(window) = &self.window {
                                if let Some(seat) = self.seat_state.seats().next() {
                                    let resize_edge = if at_top_corner && at_left_corner {
                                        Some(ResizeEdge::TopLeft)
                                    } else if at_top_corner && at_right_corner {
                                        Some(ResizeEdge::TopRight)
                                    } else if at_bottom_corner && at_left_corner {
                                        Some(ResizeEdge::BottomLeft)
                                    } else if at_bottom_corner && at_right_corner {
                                        Some(ResizeEdge::BottomRight)
                                    } else if at_top {
                                        Some(ResizeEdge::Top)
                                    } else if at_bottom {
                                        Some(ResizeEdge::Bottom)
                                    } else if at_left {
                                        Some(ResizeEdge::Left)
                                    } else if at_right {
                                        Some(ResizeEdge::Right)
                                    } else {
                                        None
                                    };
                                    
                                    if let Some(edge) = resize_edge {
                                        window.resize(&seat, serial, edge);
                                    } else if y < 32.0 {
                                        window.move_(&seat, serial);
                                    }
                                }
                            }
                        }
                    }
                }
                Release { .. } => {}
                Axis { .. } => {}
            }
        }
    }
}
