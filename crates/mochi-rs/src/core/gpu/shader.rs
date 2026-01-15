use glam::{Vec2, Vec3, Vec4};

pub struct ShaderProgram {
    pub id: u32,
    pub vertex_source: String,
    pub fragment_source: String,
}

impl ShaderProgram {
    pub fn new(vertex_source: String, fragment_source: String) -> Self {
        Self {
            id: 0,
            vertex_source,
            fragment_source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderEffect {
    pub effect_type: ShaderEffectType,
    pub parameters: ShaderParameters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderEffectType {
    Blur,
    Glow,
    Shadow,
    Gradient,
    RoundedRect,
    ColorMatrix,
    Desaturate,
    Brightness,
    Contrast,
}

#[derive(Debug, Clone)]
pub struct ShaderParameters {
    pub blur_radius: f32,
    pub glow_intensity: f32,
    pub shadow_offset: Vec2,
    pub shadow_blur: f32,
    pub gradient_start: Vec4,
    pub gradient_end: Vec4,
    pub gradient_angle: f32,
    pub corner_radius: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

impl Default for ShaderParameters {
    fn default() -> Self {
        Self {
            blur_radius: 5.0,
            glow_intensity: 1.0,
            shadow_offset: Vec2::new(2.0, 2.0),
            shadow_blur: 4.0,
            gradient_start: Vec4::new(1.0, 1.0, 1.0, 1.0),
            gradient_end: Vec4::new(0.0, 0.0, 0.0, 1.0),
            gradient_angle: 0.0,
            corner_radius: 8.0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

impl ShaderEffect {
    pub fn blur(radius: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.blur_radius = radius;
        Self {
            effect_type: ShaderEffectType::Blur,
            parameters: params,
        }
    }

    pub fn glow(intensity: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.glow_intensity = intensity;
        Self {
            effect_type: ShaderEffectType::Glow,
            parameters: params,
        }
    }

    pub fn shadow(offset: Vec2, blur: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.shadow_offset = offset;
        params.shadow_blur = blur;
        Self {
            effect_type: ShaderEffectType::Shadow,
            parameters: params,
        }
    }

    pub fn gradient(start: Vec4, end: Vec4, angle: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.gradient_start = start;
        params.gradient_end = end;
        params.gradient_angle = angle;
        Self {
            effect_type: ShaderEffectType::Gradient,
            parameters: params,
        }
    }

    pub fn rounded_rect(radius: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.corner_radius = radius;
        Self {
            effect_type: ShaderEffectType::RoundedRect,
            parameters: params,
        }
    }

    pub fn brightness(value: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.brightness = value;
        Self {
            effect_type: ShaderEffectType::Brightness,
            parameters: params,
        }
    }

    pub fn contrast(value: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.contrast = value;
        Self {
            effect_type: ShaderEffectType::Contrast,
            parameters: params,
        }
    }

    pub fn desaturate(amount: f32) -> Self {
        let mut params = ShaderParameters::default();
        params.saturation = 1.0 - amount;
        Self {
            effect_type: ShaderEffectType::Desaturate,
            parameters: params,
        }
    }
}

// Built-in shader sources
pub mod shaders {
    pub const BASIC_VERTEX: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec2 aTexCoord;

out vec4 vertexColor;
out vec2 texCoord;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    vertexColor = aColor;
    texCoord = aTexCoord;
}
"#;

    pub const BASIC_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

void main() {
    FragColor = vertexColor;
}
"#;

    pub const BLUR_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform float blurRadius;
uniform vec2 resolution;

void main() {
    vec4 color = vec4(0.0);
    float total = 0.0;
    
    for (float x = -blurRadius; x <= blurRadius; x++) {
        for (float y = -blurRadius; y <= blurRadius; y++) {
            vec2 offset = vec2(x, y) / resolution;
            float weight = 1.0 / (1.0 + length(vec2(x, y)));
            color += vertexColor * weight;
            total += weight;
        }
    }
    
    FragColor = color / total;
}
"#;

    pub const GLOW_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform float glowIntensity;

void main() {
    vec4 color = vertexColor;
    float luminance = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    vec3 glow = color.rgb * glowIntensity * luminance;
    FragColor = vec4(color.rgb + glow, color.a);
}
"#;

    pub const GRADIENT_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform vec4 gradientStart;
uniform vec4 gradientEnd;
uniform float gradientAngle;

void main() {
    float angle = radians(gradientAngle);
    vec2 dir = vec2(cos(angle), sin(angle));
    float t = dot(texCoord - 0.5, dir) + 0.5;
    t = clamp(t, 0.0, 1.0);
    
    vec4 gradient = mix(gradientStart, gradientEnd, t);
    FragColor = gradient * vertexColor;
}
"#;

    pub const ROUNDED_RECT_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform float cornerRadius;
uniform vec2 resolution;

void main() {
    vec2 pos = texCoord * resolution;
    vec2 halfRes = resolution * 0.5;
    
    // Distance from corners
    vec2 d = abs(pos - halfRes) - (halfRes - cornerRadius);
    float dist = length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
    
    float alpha = 1.0 - smoothstep(cornerRadius - 1.0, cornerRadius, dist);
    FragColor = vec4(vertexColor.rgb, vertexColor.a * alpha);
}
"#;

    pub const BRIGHTNESS_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform float brightness;

void main() {
    vec4 color = vertexColor;
    FragColor = vec4(color.rgb * brightness, color.a);
}
"#;

    pub const CONTRAST_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform float contrast;

void main() {
    vec4 color = vertexColor;
    vec3 adjusted = (color.rgb - 0.5) * contrast + 0.5;
    FragColor = vec4(adjusted, color.a);
}
"#;

    pub const DESATURATE_FRAGMENT: &str = r#"
#version 330 core
in vec4 vertexColor;
in vec2 texCoord;
out vec4 FragColor;

uniform float saturation;

void main() {
    vec4 color = vertexColor;
    float gray = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    vec3 desaturated = mix(vec3(gray), color.rgb, saturation);
    FragColor = vec4(desaturated, color.a);
}
"#;
}
