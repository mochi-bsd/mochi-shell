use glam::{Vec2, Vec3, Vec4};
use std::collections::HashMap;

/// GLSL-like shader context for software rendering
pub struct ShaderContext {
    pub frag_coord: Vec2,
    pub resolution: Vec2,
    pub time: f32,
    pub uniforms: HashMap<String, UniformValue>,
}

#[derive(Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}

impl ShaderContext {
    pub fn new(resolution: Vec2) -> Self {
        Self {
            frag_coord: Vec2::ZERO,
            resolution,
            time: 0.0,
            uniforms: HashMap::new(),
        }
    }

    pub fn set_uniform(&mut self, name: &str, value: UniformValue) {
        self.uniforms.insert(name.to_string(), value);
    }

    pub fn get_float(&self, name: &str) -> f32 {
        match self.uniforms.get(name) {
            Some(UniformValue::Float(v)) => *v,
            _ => 0.0,
        }
    }

    pub fn get_vec2(&self, name: &str) -> Vec2 {
        match self.uniforms.get(name) {
            Some(UniformValue::Vec2(v)) => *v,
            _ => Vec2::ZERO,
        }
    }

    pub fn get_vec3(&self, name: &str) -> Vec3 {
        match self.uniforms.get(name) {
            Some(UniformValue::Vec3(v)) => *v,
            _ => Vec3::ZERO,
        }
    }

    pub fn get_vec4(&self, name: &str) -> Vec4 {
        match self.uniforms.get(name) {
            Some(UniformValue::Vec4(v)) => *v,
            _ => Vec4::ZERO,
        }
    }
}

/// Trait for GLSL-like fragment shaders
pub trait FragmentShader: Send + Sync {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4;
}

/// GLSL helper functions
pub mod glsl {
    use glam::{Vec2, Vec3, Vec4};

    pub fn mix(a: Vec4, b: Vec4, t: f32) -> Vec4 {
        a * (1.0 - t) + b * t
    }

    pub fn mix_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
        a * (1.0 - t) + b * t
    }

    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn length(v: Vec2) -> f32 {
        v.length()
    }

    pub fn distance(a: Vec2, b: Vec2) -> f32 {
        (a - b).length()
    }

    pub fn dot(a: Vec2, b: Vec2) -> f32 {
        a.dot(b)
    }

    pub fn normalize(v: Vec2) -> Vec2 {
        v.normalize_or_zero()
    }

    pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
        x.clamp(min, max)
    }

    pub fn fract(x: f32) -> f32 {
        x - x.floor()
    }

    pub fn sin(x: f32) -> f32 {
        x.sin()
    }

    pub fn cos(x: f32) -> f32 {
        x.cos()
    }

    pub fn abs(x: f32) -> f32 {
        x.abs()
    }

    pub fn pow(x: f32, y: f32) -> f32 {
        x.powf(y)
    }

    pub fn step(edge: f32, x: f32) -> f32 {
        if x < edge { 0.0 } else { 1.0 }
    }

    pub fn max(a: f32, b: f32) -> f32 {
        a.max(b)
    }

    pub fn min(a: f32, b: f32) -> f32 {
        a.min(b)
    }
}

// Built-in shader implementations

/// Simple gradient shader
pub struct GradientShader {
    pub color_start: Vec4,
    pub color_end: Vec4,
    pub angle: f32,
}

impl FragmentShader for GradientShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let angle_rad = self.angle.to_radians();
        let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
        let t = glsl::dot(uv - Vec2::splat(0.5), dir) + 0.5;
        let t = glsl::clamp(t, 0.0, 1.0);
        glsl::mix(self.color_start, self.color_end, t)
    }
}

/// Radial gradient shader
pub struct RadialGradientShader {
    pub color_center: Vec4,
    pub color_edge: Vec4,
    pub center: Vec2,
    pub radius: f32,
}

impl FragmentShader for RadialGradientShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let dist = glsl::distance(uv, self.center);
        let t = glsl::clamp(dist / self.radius, 0.0, 1.0);
        glsl::mix(self.color_center, self.color_edge, t)
    }
}

/// Glow shader
pub struct GlowShader {
    pub base_color: Vec4,
    pub glow_intensity: f32,
}

impl FragmentShader for GlowShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let color = self.base_color;
        let luminance = color.x * 0.299 + color.y * 0.587 + color.z * 0.114;
        let glow = Vec3::new(color.x, color.y, color.z) * self.glow_intensity * luminance;
        Vec4::new(
            (color.x + glow.x).min(1.0),
            (color.y + glow.y).min(1.0),
            (color.z + glow.z).min(1.0),
            color.w,
        )
    }
}

/// Rounded rectangle SDF shader
pub struct RoundedRectShader {
    pub color: Vec4,
    pub rect_pos: Vec2,
    pub rect_size: Vec2,
    pub corner_radius: f32,
}

impl FragmentShader for RoundedRectShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let pos = ctx.frag_coord;
        let half_size = self.rect_size * 0.5;
        let center = self.rect_pos + half_size;
        
        // Distance from center
        let d = (pos - center).abs() - (half_size - Vec2::splat(self.corner_radius));
        let dist = glsl::length(Vec2::new(glsl::max(d.x, 0.0), glsl::max(d.y, 0.0))) 
                  + glsl::min(glsl::max(d.x, d.y), 0.0);
        
        // Anti-aliased edge
        let alpha = 1.0 - glsl::smoothstep(self.corner_radius - 1.0, self.corner_radius, dist);
        Vec4::new(self.color.x, self.color.y, self.color.z, self.color.w * alpha)
    }
}

/// Blur shader (box blur approximation)
pub struct BlurShader {
    pub base_color: Vec4,
    pub blur_radius: f32,
}

impl FragmentShader for BlurShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        // Simple blur approximation - in real implementation would sample texture
        let uv = ctx.frag_coord / ctx.resolution;
        let center_dist = glsl::distance(uv, Vec2::splat(0.5));
        let blur_factor = (self.blur_radius * center_dist * 0.1).min(0.3);
        
        let color = self.base_color;
        Vec4::new(
            color.x * (1.0 - blur_factor),
            color.y * (1.0 - blur_factor),
            color.z * (1.0 - blur_factor),
            color.w,
        )
    }
}

/// Wave distortion shader
pub struct WaveShader {
    pub base_color: Vec4,
    pub amplitude: f32,
    pub frequency: f32,
}

impl FragmentShader for WaveShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let wave = glsl::sin(uv.y * self.frequency + ctx.time) * self.amplitude;
        let distorted_x = uv.x + wave;
        
        // Color based on distortion
        let t = glsl::fract(distorted_x);
        let color = self.base_color;
        Vec4::new(
            color.x * (0.8 + t * 0.2),
            color.y * (0.8 + t * 0.2),
            color.z * (0.8 + t * 0.2),
            color.w,
        )
    }
}

/// Noise-based shader
pub struct NoiseShader {
    pub base_color: Vec4,
    pub scale: f32,
}

impl NoiseShader {
    fn hash(&self, p: Vec2) -> f32 {
        let p3 = glsl::fract(p.x * 0.1031 + p.y * 0.1030);
        let p3 = p3 + glsl::dot(Vec2::new(p3, p3), Vec2::new(p3 + 33.33, p3 + 33.33));
        glsl::fract((p3 + p3) * p3)
    }

    fn noise(&self, p: Vec2) -> f32 {
        let i = Vec2::new(p.x.floor(), p.y.floor());
        let f = Vec2::new(glsl::fract(p.x), glsl::fract(p.y));
        
        let a = self.hash(i);
        let b = self.hash(i + Vec2::new(1.0, 0.0));
        let c = self.hash(i + Vec2::new(0.0, 1.0));
        let d = self.hash(i + Vec2::new(1.0, 1.0));
        
        let u = f * f * (Vec2::splat(3.0) - Vec2::splat(2.0) * f);
        
        a * (1.0 - u.x) * (1.0 - u.y) +
        b * u.x * (1.0 - u.y) +
        c * (1.0 - u.x) * u.y +
        d * u.x * u.y
    }
}

impl FragmentShader for NoiseShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let n = self.noise(uv * self.scale);
        
        let color = self.base_color;
        Vec4::new(
            color.x * (0.7 + n * 0.3),
            color.y * (0.7 + n * 0.3),
            color.z * (0.7 + n * 0.3),
            color.w,
        )
    }
}

/// Chromatic aberration shader
pub struct ChromaticAberrationShader {
    pub base_color: Vec4,
    pub offset: f32,
}

impl FragmentShader for ChromaticAberrationShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let center = Vec2::splat(0.5);
        let dir = (uv - center).normalize_or_zero();
        
        let r_offset = dir * self.offset;
        let b_offset = dir * -self.offset;
        
        let color = self.base_color;
        Vec4::new(
            color.x * (1.0 + r_offset.length()),
            color.y,
            color.z * (1.0 + b_offset.length()),
            color.w,
        )
    }
}

/// Box blur shader (multi-pass approximation)
pub struct BoxBlurShader {
    pub base_color: Vec4,
    pub blur_radius: f32,
    pub samples: i32,
}

impl FragmentShader for BoxBlurShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let pixel_size = 1.0 / ctx.resolution;
        
        let mut color_sum = Vec4::ZERO;
        let mut weight_sum = 0.0;
        
        let samples = self.samples.max(1).min(16); // Limit samples for performance
        let radius = self.blur_radius;
        
        for x in -samples..=samples {
            for y in -samples..=samples {
                let offset = Vec2::new(x as f32, y as f32) * pixel_size * radius;
                let sample_uv = uv + offset;
                
                // Only sample if within bounds
                if sample_uv.x >= 0.0 && sample_uv.x <= 1.0 && sample_uv.y >= 0.0 && sample_uv.y <= 1.0 {
                    let weight = 1.0 / (1.0 + (x * x + y * y) as f32);
                    color_sum += self.base_color * weight;
                    weight_sum += weight;
                }
            }
        }
        
        if weight_sum > 0.0 {
            color_sum / weight_sum
        } else {
            self.base_color
        }
    }
}

/// Gaussian blur shader (approximation)
pub struct GaussianBlurShader {
    pub base_color: Vec4,
    pub blur_radius: f32,
    pub sigma: f32,
}

impl GaussianBlurShader {
    fn gaussian(&self, x: f32, y: f32) -> f32 {
        let sigma_sq = self.sigma * self.sigma;
        let coefficient = 1.0 / (2.0 * std::f32::consts::PI * sigma_sq);
        let exponent = -(x * x + y * y) / (2.0 * sigma_sq);
        coefficient * exponent.exp()
    }
}

impl FragmentShader for GaussianBlurShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let pixel_size = 1.0 / ctx.resolution;
        
        let mut color_sum = Vec4::ZERO;
        let mut weight_sum = 0.0;
        
        let samples = (self.blur_radius * 2.0) as i32;
        let samples = samples.max(1).min(12);
        
        for x in -samples..=samples {
            for y in -samples..=samples {
                let offset = Vec2::new(x as f32, y as f32) * pixel_size;
                let weight = self.gaussian(x as f32, y as f32);
                
                color_sum += self.base_color * weight;
                weight_sum += weight;
            }
        }
        
        if weight_sum > 0.0 {
            color_sum / weight_sum
        } else {
            self.base_color
        }
    }
}

/// Drop shadow shader
pub struct DropShadowShader {
    pub base_color: Vec4,
    pub shadow_color: Vec4,
    pub shadow_offset: Vec2,
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
}

impl FragmentShader for DropShadowShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let center = Vec2::splat(0.5);
        
        // Calculate distance from center
        let dist_from_center = (uv - center).length();
        
        // Create a soft circular shape for the base
        let base_shape = 1.0 - glsl::smoothstep(0.3, 0.5, dist_from_center);
        
        // Shadow position (offset from base)
        let shadow_uv = uv - (self.shadow_offset / (ctx.resolution * 10.0));
        let shadow_dist = (shadow_uv - center).length();
        
        // Shadow shape with blur
        let shadow_radius = 0.5 + self.shadow_blur / 100.0;
        let shadow_shape = 1.0 - glsl::smoothstep(0.3, shadow_radius, shadow_dist);
        let shadow_alpha = shadow_shape * self.shadow_opacity;
        
        // Composite: shadow behind base
        if base_shape > 0.01 {
            // Show base color where shape exists
            Vec4::new(
                self.base_color.x,
                self.base_color.y,
                self.base_color.z,
                self.base_color.w * base_shape,
            )
        } else if shadow_alpha > 0.01 {
            // Show shadow where base doesn't exist
            Vec4::new(
                self.shadow_color.x,
                self.shadow_color.y,
                self.shadow_color.z,
                shadow_alpha,
            )
        } else {
            // Transparent
            Vec4::ZERO
        }
    }
}

/// Inner shadow shader
pub struct InnerShadowShader {
    pub base_color: Vec4,
    pub shadow_color: Vec4,
    pub shadow_offset: Vec2,
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
}

impl FragmentShader for InnerShadowShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let center = Vec2::splat(0.5);
        
        // Distance from edge (inverted for inner shadow)
        let edge_dist = glsl::min(
            glsl::min(uv.x, 1.0 - uv.x),
            glsl::min(uv.y, 1.0 - uv.y),
        );
        
        // Shadow strength based on distance from edge
        let shadow_strength = 1.0 - glsl::smoothstep(0.0, self.shadow_blur / 100.0, edge_dist);
        let shadow_alpha = shadow_strength * self.shadow_opacity;
        
        // Blend shadow with base color
        let r = self.base_color.x * (1.0 - shadow_alpha) + self.shadow_color.x * shadow_alpha;
        let g = self.base_color.y * (1.0 - shadow_alpha) + self.shadow_color.y * shadow_alpha;
        let b = self.base_color.z * (1.0 - shadow_alpha) + self.shadow_color.z * shadow_alpha;
        
        Vec4::new(r, g, b, self.base_color.w)
    }
}

/// Directional blur (motion blur effect)
pub struct DirectionalBlurShader {
    pub base_color: Vec4,
    pub direction: Vec2,
    pub blur_strength: f32,
    pub samples: i32,
}

impl FragmentShader for DirectionalBlurShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let pixel_size = 1.0 / ctx.resolution;
        
        let mut color_sum = Vec4::ZERO;
        let samples = self.samples.max(1).min(16);
        
        for i in 0..samples {
            let t = i as f32 / samples as f32;
            let offset = self.direction * pixel_size * self.blur_strength * (t - 0.5);
            color_sum += self.base_color;
        }
        
        color_sum / samples as f32
    }
}
