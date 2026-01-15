mod core;

pub use core::canvas::{Canvas, RenderBackend};
pub use core::color::Color;
pub use core::dialog::{dialog, Dialog, DialogButton, DialogButtonStyle};
pub use core::text::TextRenderer;
pub use core::ui::{
    card, container, text, titlebar, vstack, Card, Container, Element, Rect, Text, Titlebar, VStack,
};
pub use core::window::{Window, WindowConfig};

// GPU rendering
pub use core::gpu::{
    GpuBackend, GpuBackendType, GpuRenderer, ShaderEffect, ShaderProgram, Vertex, SoftwareBackend,
    Effect, EffectType, EffectParams, EffectStack, RenderGraph, RenderNode, BlendMode,
    execute_render_graph,
};

// GLSL-like shader system
pub use core::gpu::glsl::{
    FragmentShader, ShaderContext, UniformValue,
    GradientShader, RadialGradientShader, GlowShader, RoundedRectShader,
    BlurShader, WaveShader, NoiseShader, ChromaticAberrationShader,
    BoxBlurShader, GaussianBlurShader, DropShadowShader, InnerShadowShader,
    DirectionalBlurShader,
};

// Re-export glam for shader development
pub use glam::{Vec2, Vec3, Vec4};
