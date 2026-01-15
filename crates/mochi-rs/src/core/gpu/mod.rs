pub mod backend;
pub mod shader;
pub mod renderer;
pub mod vertex;
pub mod glsl;
pub mod ffi;
pub mod effects;
pub mod passes_ffi;

pub use backend::{GpuBackend, GpuBackendType, SoftwareBackend};
pub use shader::{ShaderEffect, ShaderProgram};
pub use renderer::GpuRenderer;
pub use vertex::Vertex;
pub use glsl::{FragmentShader, ShaderContext, UniformValue};
pub use ffi::{GpuContext as NativeGpuContext, GpuDeviceInfo};
pub use effects::{Effect, EffectType, EffectParams, EffectStack, RenderGraph, RenderNode, BlendMode};
pub use passes_ffi::execute_render_graph;
