use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uint, c_void};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendType {
    None = 0,
    Vulkan = 1,
    OpenGL = 2,
    OpenGLES = 3,
}

#[repr(C)]
pub struct GpuDeviceInfo {
    pub backend_type: GpuBackendType,
    pub device_name: [c_char; 256],
    pub vendor_name: [c_char; 128],
    pub driver_version: [c_char; 64],
    pub max_texture_size: c_uint,
    pub supports_compute: bool,
}

impl GpuDeviceInfo {
    pub fn device_name_str(&self) -> String {
        unsafe {
            CStr::from_ptr(self.device_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn vendor_name_str(&self) -> String {
        unsafe {
            CStr::from_ptr(self.vendor_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn driver_version_str(&self) -> String {
        unsafe {
            CStr::from_ptr(self.driver_version.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
}

// Opaque pointer to C GPU context
#[repr(C)]
pub struct GpuContextHandle {
    _private: [u8; 0],
}

pub type GpuShader = c_uint;
pub type GpuBuffer = c_uint;
pub type GpuTexture = c_uint;

extern "C" {
    // Context management
    pub fn gpu_context_create(width: c_uint, height: c_uint) -> *mut GpuContextHandle;
    pub fn gpu_context_destroy(ctx: *mut GpuContextHandle);
    pub fn gpu_context_is_valid(ctx: *const GpuContextHandle) -> bool;
    pub fn gpu_context_get_backend(ctx: *const GpuContextHandle) -> GpuBackendType;
    pub fn gpu_context_get_device_info(ctx: *const GpuContextHandle, info: *mut GpuDeviceInfo);

    // Rendering operations
    pub fn gpu_context_clear(ctx: *mut GpuContextHandle, r: c_float, g: c_float, b: c_float, a: c_float);
    pub fn gpu_context_viewport(ctx: *mut GpuContextHandle, x: c_int, y: c_int, width: c_uint, height: c_uint);
    pub fn gpu_context_present(ctx: *mut GpuContextHandle);

    // Shader operations
    pub fn gpu_context_create_shader(
        ctx: *mut GpuContextHandle,
        vertex_src: *const c_char,
        fragment_src: *const c_char,
    ) -> GpuShader;
    pub fn gpu_context_use_shader(ctx: *mut GpuContextHandle, shader: GpuShader);
    pub fn gpu_context_delete_shader(ctx: *mut GpuContextHandle, shader: GpuShader);

    // Uniform operations
    pub fn gpu_context_set_uniform_float(ctx: *mut GpuContextHandle, name: *const c_char, value: c_float);
    pub fn gpu_context_set_uniform_vec2(ctx: *mut GpuContextHandle, name: *const c_char, x: c_float, y: c_float);
    pub fn gpu_context_set_uniform_vec3(ctx: *mut GpuContextHandle, name: *const c_char, x: c_float, y: c_float, z: c_float);
    pub fn gpu_context_set_uniform_vec4(ctx: *mut GpuContextHandle, name: *const c_char, x: c_float, y: c_float, z: c_float, w: c_float);
    pub fn gpu_context_set_uniform_mat4(ctx: *mut GpuContextHandle, name: *const c_char, matrix: *const c_float);

    // Buffer operations
    pub fn gpu_context_create_buffer(ctx: *mut GpuContextHandle, data: *const c_float, size: c_uint) -> GpuBuffer;
    pub fn gpu_context_bind_buffer(ctx: *mut GpuContextHandle, buffer: GpuBuffer);
    pub fn gpu_context_delete_buffer(ctx: *mut GpuContextHandle, buffer: GpuBuffer);

    // Texture operations
    pub fn gpu_context_create_texture(ctx: *mut GpuContextHandle, width: c_uint, height: c_uint, data: *const u8) -> GpuTexture;
    pub fn gpu_context_bind_texture(ctx: *mut GpuContextHandle, texture: GpuTexture, slot: c_uint);
    pub fn gpu_context_delete_texture(ctx: *mut GpuContextHandle, texture: GpuTexture);

    // Draw operations
    pub fn gpu_context_draw_arrays(ctx: *mut GpuContextHandle, mode: c_uint, first: c_int, count: c_int);
    pub fn gpu_context_draw_elements(ctx: *mut GpuContextHandle, mode: c_uint, count: c_int, indices: *const c_uint);
}

// Safe Rust wrapper
pub struct GpuContext {
    handle: *mut GpuContextHandle,
}

impl GpuContext {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        unsafe {
            let handle = gpu_context_create(width, height);
            if handle.is_null() || !gpu_context_is_valid(handle) {
                if !handle.is_null() {
                    gpu_context_destroy(handle);
                }
                return None;
            }
            Some(Self { handle })
        }
    }

    pub fn get_backend(&self) -> GpuBackendType {
        unsafe { gpu_context_get_backend(self.handle) }
    }

    pub fn get_device_info(&self) -> GpuDeviceInfo {
        unsafe {
            let mut info = std::mem::zeroed();
            gpu_context_get_device_info(self.handle, &mut info);
            info
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gpu_context_clear(self.handle, r, g, b, a);
        }
    }

    pub fn viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            gpu_context_viewport(self.handle, x, y, width, height);
        }
    }

    pub fn present(&mut self) {
        unsafe {
            gpu_context_present(self.handle);
        }
    }

    pub fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> Option<GpuShader> {
        unsafe {
            let vertex_cstr = CString::new(vertex_src).ok()?;
            let fragment_cstr = CString::new(fragment_src).ok()?;
            let shader = gpu_context_create_shader(
                self.handle,
                vertex_cstr.as_ptr(),
                fragment_cstr.as_ptr(),
            );
            if shader == 0 {
                None
            } else {
                Some(shader)
            }
        }
    }

    pub fn use_shader(&mut self, shader: GpuShader) {
        unsafe {
            gpu_context_use_shader(self.handle, shader);
        }
    }

    pub fn delete_shader(&mut self, shader: GpuShader) {
        unsafe {
            gpu_context_delete_shader(self.handle, shader);
        }
    }

    pub fn set_uniform_float(&mut self, name: &str, value: f32) {
        unsafe {
            if let Ok(name_cstr) = CString::new(name) {
                gpu_context_set_uniform_float(self.handle, name_cstr.as_ptr(), value);
            }
        }
    }

    pub fn set_uniform_vec2(&mut self, name: &str, x: f32, y: f32) {
        unsafe {
            if let Ok(name_cstr) = CString::new(name) {
                gpu_context_set_uniform_vec2(self.handle, name_cstr.as_ptr(), x, y);
            }
        }
    }

    pub fn set_uniform_vec3(&mut self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            if let Ok(name_cstr) = CString::new(name) {
                gpu_context_set_uniform_vec3(self.handle, name_cstr.as_ptr(), x, y, z);
            }
        }
    }

    pub fn set_uniform_vec4(&mut self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            if let Ok(name_cstr) = CString::new(name) {
                gpu_context_set_uniform_vec4(self.handle, name_cstr.as_ptr(), x, y, z, w);
            }
        }
    }

    pub fn set_uniform_mat4(&mut self, name: &str, matrix: &[f32; 16]) {
        unsafe {
            if let Ok(name_cstr) = CString::new(name) {
                gpu_context_set_uniform_mat4(self.handle, name_cstr.as_ptr(), matrix.as_ptr());
            }
        }
    }

    pub fn create_buffer(&mut self, data: &[f32]) -> Option<GpuBuffer> {
        unsafe {
            let buffer = gpu_context_create_buffer(
                self.handle,
                data.as_ptr(),
                (data.len() * std::mem::size_of::<f32>()) as u32,
            );
            if buffer == 0 {
                None
            } else {
                Some(buffer)
            }
        }
    }

    pub fn bind_buffer(&mut self, buffer: GpuBuffer) {
        unsafe {
            gpu_context_bind_buffer(self.handle, buffer);
        }
    }

    pub fn delete_buffer(&mut self, buffer: GpuBuffer) {
        unsafe {
            gpu_context_delete_buffer(self.handle, buffer);
        }
    }

    pub fn create_texture(&mut self, width: u32, height: u32, data: &[u8]) -> Option<GpuTexture> {
        unsafe {
            let texture = gpu_context_create_texture(self.handle, width, height, data.as_ptr());
            if texture == 0 {
                None
            } else {
                Some(texture)
            }
        }
    }

    pub fn bind_texture(&mut self, texture: GpuTexture, slot: u32) {
        unsafe {
            gpu_context_bind_texture(self.handle, texture, slot);
        }
    }

    pub fn delete_texture(&mut self, texture: GpuTexture) {
        unsafe {
            gpu_context_delete_texture(self.handle, texture);
        }
    }

    pub fn draw_arrays(&mut self, mode: u32, first: i32, count: i32) {
        unsafe {
            gpu_context_draw_arrays(self.handle, mode, first, count);
        }
    }

    pub fn draw_elements(&mut self, mode: u32, indices: &[u32]) {
        unsafe {
            gpu_context_draw_elements(self.handle, mode, indices.len() as i32, indices.as_ptr());
        }
    }

    pub fn handle(&self) -> *mut GpuContextHandle {
        self.handle
    }
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                gpu_context_destroy(self.handle);
            }
        }
    }
}

unsafe impl Send for GpuContext {}
unsafe impl Sync for GpuContext {}
