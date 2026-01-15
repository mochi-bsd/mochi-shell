# Hanami Compositor

A Wayland compositor for the Mochi Desktop Environment, built with Rust and Smithay.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Hanami Compositor                      │
├─────────────────────────────────────────────────────────┤
│  Rust Layer (Smithay)                                   │
│  • Window management                                     │
│  • Wayland protocol handling                            │
│  • XDG Shell, Layer Shell                               │
│  • Input event routing                                   │
│  • Workspace management                                  │
├─────────────────────────────────────────────────────────┤
│  C Layer (GPU Backend)                                   │
│  • OpenGL ES 3.0 rendering                              │
│  • DRM/KMS output management                            │
│  • libinput integration                                  │
│  • Hardware acceleration                                 │
└─────────────────────────────────────────────────────────┘
```

## Current Status

🚧 **Work in Progress** 🚧

The Hanami compositor is currently under active development. The basic structure is in place, but full functionality is not yet implemented.

### Completed
- ✅ Project structure
- ✅ Build system (Cargo + cc)
- ✅ Logging infrastructure
- ✅ GPU backend FFI bindings

### In Progress
- 🔄 Smithay integration
- 🔄 Window management
- 🔄 XDG Shell protocol
- 🔄 Layer Shell protocol

### Planned
- ⏳ Tiling window management
- ⏳ Workspace management
- ⏳ Window animations
- ⏳ Multi-monitor support
- ⏳ Screenshot/screencast support

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Dependencies

- **Smithay 0.7** - Wayland compositor framework
- **slog** - Structured logging
- **calloop** - Event loop
- **EGL/GLESv2** - OpenGL rendering
- **libinput** - Input device handling
- **libdrm** - Direct Rendering Manager

## Configuration

Configuration will be done through:
1. TOML config files in `~/.config/mochi/compositor.toml`
2. Rust API for programmatic configuration
3. IPC for runtime configuration

## License

Part of the Mochi Desktop Environment project.
