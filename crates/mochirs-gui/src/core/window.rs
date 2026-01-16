use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        SeatHandler, SeatState,
    },
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
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, EventQueue, QueueHandle,
};

use crate::core::canvas::Canvas;
use crate::core::color::Color;

// Debug logging macro
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("MOCHI_DEBUG").is_ok() {
            eprintln!("[MOCHI DEBUG] {}", format!($($arg)*));
        }
    };
}

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub decorations: bool,
    pub transparent: bool,
    pub draggable: bool,
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
            transparent: false,
            draggable: true,
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
    is_resizing: bool,
    last_resize_time: std::time::Instant,
    resize_debounce_ms: u64,
    // Buffer pooling optimization
    last_buffer_size: Option<(u32, u32)>,
    // Window configuration
    transparent: bool,
    draggable: bool,
}

pub struct Window {
    event_queue: EventQueue<AppState>,
    state: AppState,
}

impl Window {
    pub fn new(config: WindowConfig) -> Result<Self, Box<dyn std::error::Error>> {
        debug_log!("Window::new() - Creating window: {}x{}", config.width, config.height);
        
        // Force LLVMpipe software rendering
        std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        std::env::set_var("GALLIUM_DRIVER", "llvmpipe");
        std::env::set_var("LP_NUM_THREADS", "4"); // Use 4 threads for better performance
        
        // Log renderer info
        let renderer = std::env::var("GALLIUM_DRIVER").unwrap_or_else(|_| "unknown".to_string());
        debug_log!("Software Renderer: LLVMpipe (forced)");
        debug_log!("GALLIUM_DRIVER={}", renderer);
        debug_log!("LIBGL_ALWAYS_SOFTWARE=1");
        
        let conn = Connection::connect_to_env()?;
        debug_log!("Connected to Wayland display");
        
        let (globals, event_queue) = registry_queue_init::<AppState>(&conn)?;
        debug_log!("Registry initialized");
        
        let qh = event_queue.handle();

        debug_log!("Binding Wayland protocols...");
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
            is_resizing: false,
            last_resize_time: std::time::Instant::now(),
            resize_debounce_ms: 150, // Wait 150ms after resize before full redraw
            last_buffer_size: None,
            transparent: config.transparent,
            draggable: config.draggable,
        };
        debug_log!("Wayland protocols bound successfully");

        let mut window = Self { event_queue, state };

        let qh = window.event_queue.handle();
        debug_log!("Creating surface...");
        let surface = window.state.compositor_state.create_surface(&qh);

        // Use client-side decorations for custom titlebar dragging
        let decorations = WindowDecorations::RequestClient;

        debug_log!("Creating XDG window...");
        let xdg_window = window
            .state
            .xdg_shell_state
            .create_window(surface, decorations, &qh);

        debug_log!("Setting window properties: title='{}', min_size={:?}", 
                   config.title, (config.min_width, config.min_height));
        xdg_window.set_title(&config.title);
        if let (Some(min_w), Some(min_h)) = (config.min_width, config.min_height) {
            xdg_window.set_min_size(Some((min_w, min_h)));
        }
        
        // If no min size is set, disable resizing by setting max size = current size
        if config.min_width.is_none() && config.min_height.is_none() {
            xdg_window.set_max_size(Some((config.width, config.height)));
        }
        
        xdg_window.commit();

        window.state.window = Some(xdg_window);
        debug_log!("Window created successfully");

        Ok(window)
    }

    pub fn on_draw<F>(&mut self, f: F)
    where
        F: FnMut(&mut Canvas) + 'static,
    {
        self.state.draw_fn = Some(Box::new(f));
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug_log!("Entering main event loop");
        debug_log!("=== Mochi Window System ===");
        debug_log!("Renderer: LLVMpipe (Mesa Software Renderer)");
        debug_log!("Backend: Wayland + Smithay Client Toolkit");
        debug_log!("Resolution: {}x{}", self.state.width, self.state.height);
        debug_log!("===========================");
        
        let mut frame_count = 0u64;
        let start_time = std::time::Instant::now();
        
        loop {
            self.event_queue.blocking_dispatch(&mut self.state)?;
            
            // Check if we need to trigger a delayed full redraw after resize
            if self.state.is_resizing {
                let elapsed = self.state.last_resize_time.elapsed();
                if elapsed.as_millis() >= self.state.resize_debounce_ms as u128 {
                    debug_log!("Triggering delayed full redraw after resize");
                    self.state.is_resizing = false;
                    if let Some(window) = &self.state.window {
                        let qh = self.event_queue.handle();
                        self.state.draw(&qh, false);
                    }
                }
            }
            
            frame_count += 1;
            if frame_count % 60 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let fps = frame_count as f64 / elapsed;
                debug_log!("Frame {}: {:.1} FPS, Size: {}x{}", 
                          frame_count, fps, self.state.width, self.state.height);
            }
        }
    }
}

impl AppState {
    fn draw(&mut self, _qh: &QueueHandle<Self>, skip_expensive: bool) {
        let draw_start = std::time::Instant::now();

        let window = match &self.window {
            Some(w) => w,
            None => {
                debug_log!("draw() called but window is None");
                return;
            }
        };

        let pool = match &mut self.pool {
            Some(p) => p,
            None => {
                debug_log!("draw() called but pool is None");
                return;
            }
        };

        let stride = self.width as i32 * 4;
        let buffer_size = (self.width * self.height * 4) as usize;
        
        // Check if buffer size changed (optimization: reuse buffer when size unchanged)
        let size_changed = self.last_buffer_size != Some((self.width, self.height));
        if size_changed {
            debug_log!("Buffer size changed, creating new buffer: {}x{} ({}KB)", 
                      self.width, self.height, buffer_size / 1024);
            self.last_buffer_size = Some((self.width, self.height));
        } else {
            debug_log!("Reusing buffer: {}x{}", self.width, self.height);
        }

        let (buffer, canvas_buffer) = pool
            .create_buffer(
                self.width as i32,
                self.height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");

        if skip_expensive {
            debug_log!("Fast draw (skipping expensive rendering)");
            // During resize, just fill with solid background color for performance
            let bg_color = if self.transparent {
                Color::TRANSPARENT
            } else {
                Color::rgb(40, 40, 40)
            };
            
            for pixel in canvas_buffer.chunks_exact_mut(4) {
                pixel[0] = bg_color.b;  // B
                pixel[1] = bg_color.g;  // G
                pixel[2] = bg_color.r;  // R
                pixel[3] = bg_color.a;  // A
            }
        } else {
            debug_log!("Full draw with effects");
            let canvas_start = std::time::Instant::now();
            
            let mut canvas = Canvas::new(canvas_buffer, self.width, self.height);

            // Clear background - use transparent if configured
            let bg_color = if self.transparent {
                Color::TRANSPARENT
            } else {
                Color::BG_PRIMARY
            };
            canvas.clear(bg_color);

            // Call user draw function
            if let Some(ref mut draw_fn) = self.draw_fn {
                draw_fn(&mut canvas);
            }
            
            let canvas_elapsed = canvas_start.elapsed();
            debug_log!("Canvas rendering took: {:.2}ms", canvas_elapsed.as_secs_f64() * 1000.0);
        }

        window.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        
        // Use damage_buffer for more efficient updates (only redraw changed regions)
        // For now, damage the entire buffer, but this could be optimized with dirty region tracking
        window
            .wl_surface()
            .damage_buffer(0, 0, self.width as i32, self.height as i32);
        window.wl_surface().commit();
        
        let draw_elapsed = draw_start.elapsed();
        debug_log!("Total draw() took: {:.2}ms", draw_elapsed.as_secs_f64() * 1000.0);
    }
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        time: u32,
    ) {
        debug_log!("frame() callback: time={}, was_resizing={}", time, self.is_resizing);
        
        // Check if enough time has passed since last resize
        if self.is_resizing {
            let elapsed = self.last_resize_time.elapsed();
            if elapsed.as_millis() < self.resize_debounce_ms as u128 {
                debug_log!("Resize debounce: waiting {}ms more", 
                          self.resize_debounce_ms as u128 - elapsed.as_millis());
                // Still resizing, request another frame callback
                if let Some(window) = &self.window {
                    window.wl_surface().frame(qh, window.wl_surface().clone());
                }
                return;
            }
            
            // Resize settled, do full redraw
            debug_log!("Resize settled - doing full redraw");
            self.is_resizing = false;
        }
        
        // Full redraw
        self.draw(qh, false);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
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
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl WindowHandler for AppState {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &XdgWindow) {
        debug_log!("Window close requested");
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        serial: u32,
    ) {
        debug_log!("configure() called: serial={}", serial);
        
        let (width, height) = configure.new_size;
        let mut size_changed = false;

        if let Some(w) = width {
            if self.width != w.get() {
                debug_log!("Width changed: {} -> {}", self.width, w.get());
                self.width = w.get();
                size_changed = true;
            }
        }
        if let Some(h) = height {
            if self.height != h.get() {
                debug_log!("Height changed: {} -> {}", self.height, h.get());
                self.height = h.get();
                size_changed = true;
            }
        }

        // Recreate pool if size changed or pool doesn't exist
        if self.pool.is_none() || size_changed {
            let pool_size = (self.width * self.height * 4) as usize;
            debug_log!("Creating new pool: {} bytes", pool_size);
            self.pool = Some(
                SlotPool::new(pool_size, &self.shm_state)
                    .expect("Failed to create pool"),
            );
        }

        // During resize, use fast draw (skip expensive rendering)
        if size_changed {
            debug_log!("Size changed - using fast draw");
            self.is_resizing = true;
            self.last_resize_time = std::time::Instant::now();
            self.draw(qh, true); // Skip expensive rendering
            
            // Don't request frame callback here - let the timer handle it
        } else {
            debug_log!("Initial configure - using full draw");
            // Initial configure or state change - do full draw
            self.draw(qh, false);
        }
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
                    // Only handle dragging if enabled, no resize support
                    if button == 0x110 && self.draggable {
                        // BTN_LEFT
                        if let Some((_, y)) = self.pointer_location {
                            if y < 32.0 {
                                if let Some(window) = &self.window {
                                    if let Some(seat) = self.seat_state.seats().next() {
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
