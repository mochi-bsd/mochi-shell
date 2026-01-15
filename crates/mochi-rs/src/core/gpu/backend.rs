use glam::{Mat4, Vec2, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendType {
    Vulkan,
    OpenGL,
    OpenGLES,
}

pub trait GpuBackend {
    fn backend_type(&self) -> GpuBackendType;
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> Result<u32, String>;
    fn use_shader(&mut self, shader_id: u32);
    fn delete_shader(&mut self, shader_id: u32);
    fn create_buffer(&mut self, data: &[f32]) -> Result<u32, String>;
    fn bind_buffer(&mut self, buffer_id: u32);
    fn delete_buffer(&mut self, buffer_id: u32);
    fn set_uniform_mat4(&mut self, name: &str, matrix: &Mat4);
    fn set_uniform_vec4(&mut self, name: &str, vector: &Vec4);
    fn set_uniform_vec3(&mut self, name: &str, vector: &Vec3);
    fn set_uniform_vec2(&mut self, name: &str, vector: &Vec2);
    fn set_uniform_float(&mut self, name: &str, value: f32);
    fn set_uniform_int(&mut self, name: &str, value: i32);
    fn draw_arrays(&mut self, mode: DrawMode, first: i32, count: i32);
    fn clear(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    Triangles,
    TriangleStrip,
    Lines,
    Points,
}

// Software fallback implementation
pub struct SoftwareBackend {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

impl SoftwareBackend {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buffer.resize((width * height * 4) as usize, 0);
    }
}

impl GpuBackend for SoftwareBackend {
    fn backend_type(&self) -> GpuBackendType {
        GpuBackendType::OpenGL // Fallback
    }

    fn create_shader(&mut self, _vertex_src: &str, _fragment_src: &str) -> Result<u32, String> {
        Ok(0) // Software rendering doesn't use shaders
    }

    fn use_shader(&mut self, _shader_id: u32) {}
    fn delete_shader(&mut self, _shader_id: u32) {}
    fn create_buffer(&mut self, _data: &[f32]) -> Result<u32, String> {
        Ok(0)
    }
    fn bind_buffer(&mut self, _buffer_id: u32) {}
    fn delete_buffer(&mut self, _buffer_id: u32) {}
    fn set_uniform_mat4(&mut self, _name: &str, _matrix: &Mat4) {}
    fn set_uniform_vec4(&mut self, _name: &str, _vector: &Vec4) {}
    fn set_uniform_vec3(&mut self, _name: &str, _vector: &Vec3) {}
    fn set_uniform_vec2(&mut self, _name: &str, _vector: &Vec2) {}
    fn set_uniform_float(&mut self, _name: &str, _value: f32) {}
    fn set_uniform_int(&mut self, _name: &str, _value: i32) {}
    fn draw_arrays(&mut self, _mode: DrawMode, _first: i32, _count: i32) {}

    fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let color = [
            (b * 255.0) as u8,
            (g * 255.0) as u8,
            (r * 255.0) as u8,
            (a * 255.0) as u8,
        ];
        for chunk in self.buffer.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    fn viewport(&mut self, _x: i32, _y: i32, width: i32, height: i32) {
        if width as u32 != self.width || height as u32 != self.height {
            self.resize(width as u32, height as u32);
        }
    }
}
