use super::backend::{DrawMode, GpuBackend};
use super::shader::{ShaderEffect, ShaderProgram, shaders};
use super::vertex::{Vertex, vertices_to_floats};
use crate::core::color::Color;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;

pub struct GpuRenderer {
    backend: Box<dyn GpuBackend>,
    shaders: HashMap<String, ShaderProgram>,
    buffers: HashMap<String, u32>,
    width: u32,
    height: u32,
    projection: Mat4,
}

impl GpuRenderer {
    pub fn new(backend: Box<dyn GpuBackend>, width: u32, height: u32) -> Self {
        let projection = Mat4::orthographic_rh(
            0.0,
            width as f32,
            height as f32,
            0.0,
            -1.0,
            1.0,
        );

        let mut renderer = Self {
            backend,
            shaders: HashMap::new(),
            buffers: HashMap::new(),
            width,
            height,
            projection,
        };

        // Load built-in shaders
        renderer.load_builtin_shaders();
        renderer
    }

    fn load_builtin_shaders(&mut self) {
        let shader_defs = vec![
            ("basic", shaders::BASIC_VERTEX, shaders::BASIC_FRAGMENT),
            ("blur", shaders::BASIC_VERTEX, shaders::BLUR_FRAGMENT),
            ("glow", shaders::BASIC_VERTEX, shaders::GLOW_FRAGMENT),
            ("gradient", shaders::BASIC_VERTEX, shaders::GRADIENT_FRAGMENT),
            ("rounded_rect", shaders::BASIC_VERTEX, shaders::ROUNDED_RECT_FRAGMENT),
            ("brightness", shaders::BASIC_VERTEX, shaders::BRIGHTNESS_FRAGMENT),
            ("contrast", shaders::BASIC_VERTEX, shaders::CONTRAST_FRAGMENT),
            ("desaturate", shaders::BASIC_VERTEX, shaders::DESATURATE_FRAGMENT),
        ];

        for (name, vertex, fragment) in shader_defs {
            if let Ok(id) = self.backend.create_shader(vertex, fragment) {
                let mut program = ShaderProgram::new(vertex.to_string(), fragment.to_string());
                program.id = id;
                self.shaders.insert(name.to_string(), program);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.projection = Mat4::orthographic_rh(
            0.0,
            width as f32,
            height as f32,
            0.0,
            -1.0,
            1.0,
        );
        self.backend.viewport(0, 0, width as i32, height as i32);
    }

    pub fn clear(&mut self, color: Color) {
        let r = color.r as f32 / 255.0;
        let g = color.g as f32 / 255.0;
        let b = color.b as f32 / 255.0;
        let a = color.a as f32 / 255.0;
        self.backend.clear(r, g, b, a);
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.draw_rect_with_effect(x, y, width, height, color, None);
    }

    pub fn draw_rect_with_effect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        effect: Option<&ShaderEffect>,
    ) {
        let color_vec = Vec4::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );

        let vertices = vec![
            Vertex::new(Vec3::new(x, y, 0.0), color_vec, Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(x + width, y, 0.0), color_vec, Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(x + width, y + height, 0.0), color_vec, Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(x, y, 0.0), color_vec, Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(x + width, y + height, 0.0), color_vec, Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(x, y + height, 0.0), color_vec, Vec2::new(0.0, 1.0)),
        ];

        self.draw_vertices(&vertices, effect);
    }

    pub fn draw_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: Color,
    ) {
        let effect = ShaderEffect::rounded_rect(radius);
        self.draw_rect_with_effect(x, y, width, height, color, Some(&effect));
    }

    pub fn draw_gradient_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        start_color: Color,
        end_color: Color,
        angle: f32,
    ) {
        let start = Vec4::new(
            start_color.r as f32 / 255.0,
            start_color.g as f32 / 255.0,
            start_color.b as f32 / 255.0,
            start_color.a as f32 / 255.0,
        );
        let end = Vec4::new(
            end_color.r as f32 / 255.0,
            end_color.g as f32 / 255.0,
            end_color.b as f32 / 255.0,
            end_color.a as f32 / 255.0,
        );

        let effect = ShaderEffect::gradient(start, end, angle);
        self.draw_rect_with_effect(x, y, width, height, start_color, Some(&effect));
    }

    fn draw_vertices(&mut self, vertices: &[Vertex], effect: Option<&ShaderEffect>) {
        let shader_name = if let Some(eff) = effect {
            match eff.effect_type {
                super::shader::ShaderEffectType::Blur => "blur",
                super::shader::ShaderEffectType::Glow => "glow",
                super::shader::ShaderEffectType::Shadow => "basic",
                super::shader::ShaderEffectType::Gradient => "gradient",
                super::shader::ShaderEffectType::RoundedRect => "rounded_rect",
                super::shader::ShaderEffectType::ColorMatrix => "basic",
                super::shader::ShaderEffectType::Desaturate => "desaturate",
                super::shader::ShaderEffectType::Brightness => "brightness",
                super::shader::ShaderEffectType::Contrast => "contrast",
            }
        } else {
            "basic"
        };

        if let Some(shader) = self.shaders.get(shader_name) {
            self.backend.use_shader(shader.id);

            // Set uniforms
            self.backend.set_uniform_mat4("projection", &self.projection);
            self.backend.set_uniform_mat4("view", &Mat4::IDENTITY);
            self.backend.set_uniform_mat4("model", &Mat4::IDENTITY);

            if let Some(eff) = effect {
                let params = &eff.parameters;
                self.backend.set_uniform_vec2("resolution", &Vec2::new(self.width as f32, self.height as f32));
                
                match eff.effect_type {
                    super::shader::ShaderEffectType::Blur => {
                        self.backend.set_uniform_float("blurRadius", params.blur_radius);
                    }
                    super::shader::ShaderEffectType::Glow => {
                        self.backend.set_uniform_float("glowIntensity", params.glow_intensity);
                    }
                    super::shader::ShaderEffectType::Gradient => {
                        self.backend.set_uniform_vec4("gradientStart", &params.gradient_start);
                        self.backend.set_uniform_vec4("gradientEnd", &params.gradient_end);
                        self.backend.set_uniform_float("gradientAngle", params.gradient_angle);
                    }
                    super::shader::ShaderEffectType::RoundedRect => {
                        self.backend.set_uniform_float("cornerRadius", params.corner_radius);
                    }
                    super::shader::ShaderEffectType::Brightness => {
                        self.backend.set_uniform_float("brightness", params.brightness);
                    }
                    super::shader::ShaderEffectType::Contrast => {
                        self.backend.set_uniform_float("contrast", params.contrast);
                    }
                    super::shader::ShaderEffectType::Desaturate => {
                        self.backend.set_uniform_float("saturation", params.saturation);
                    }
                    _ => {}
                }
            }

            // Create and bind buffer
            let floats = vertices_to_floats(vertices);
            if let Ok(buffer_id) = self.backend.create_buffer(&floats) {
                self.backend.bind_buffer(buffer_id);
                self.backend.draw_arrays(DrawMode::Triangles, 0, vertices.len() as i32);
                self.backend.delete_buffer(buffer_id);
            }
        }
    }

    pub fn begin_frame(&mut self) {
        self.backend.viewport(0, 0, self.width as i32, self.height as i32);
    }

    pub fn end_frame(&mut self) {
        // Flush any pending operations
    }
}
