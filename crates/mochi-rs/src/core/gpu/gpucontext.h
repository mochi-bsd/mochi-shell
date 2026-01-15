#ifndef MOCHI_GPU_CONTEXT_H
#define MOCHI_GPU_CONTEXT_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// GPU backend types
typedef enum {
    GPU_BACKEND_NONE = 0,
    GPU_BACKEND_VULKAN = 1,
    GPU_BACKEND_OPENGL = 2,
    GPU_BACKEND_OPENGLES = 3,
} GpuBackendType;

// GPU context handle (opaque pointer)
typedef struct GpuContext GpuContext;

// Shader handle
typedef uint32_t GpuShader;

// Buffer handle
typedef uint32_t GpuBuffer;

// Texture handle
typedef uint32_t GpuTexture;

// GPU device info
typedef struct {
    GpuBackendType backend_type;
    char device_name[256];
    char vendor_name[128];
    char driver_version[64];
    uint32_t max_texture_size;
    bool supports_compute;
} GpuDeviceInfo;

// Initialization and cleanup
GpuContext* gpu_context_create(uint32_t width, uint32_t height);
void gpu_context_destroy(GpuContext* ctx);
bool gpu_context_is_valid(const GpuContext* ctx);
GpuBackendType gpu_context_get_backend(const GpuContext* ctx);
void gpu_context_get_device_info(const GpuContext* ctx, GpuDeviceInfo* info);

// Rendering operations
void gpu_context_clear(GpuContext* ctx, float r, float g, float b, float a);
void gpu_context_viewport(GpuContext* ctx, int32_t x, int32_t y, uint32_t width, uint32_t height);
void gpu_context_present(GpuContext* ctx);

// Shader operations
GpuShader gpu_context_create_shader(GpuContext* ctx, const char* vertex_src, const char* fragment_src);
void gpu_context_use_shader(GpuContext* ctx, GpuShader shader);
void gpu_context_delete_shader(GpuContext* ctx, GpuShader shader);

// Uniform operations
void gpu_context_set_uniform_float(GpuContext* ctx, const char* name, float value);
void gpu_context_set_uniform_vec2(GpuContext* ctx, const char* name, float x, float y);
void gpu_context_set_uniform_vec3(GpuContext* ctx, const char* name, float x, float y, float z);
void gpu_context_set_uniform_vec4(GpuContext* ctx, const char* name, float x, float y, float z, float w);
void gpu_context_set_uniform_mat4(GpuContext* ctx, const char* name, const float* matrix);

// Buffer operations
GpuBuffer gpu_context_create_buffer(GpuContext* ctx, const float* data, uint32_t size);
void gpu_context_bind_buffer(GpuContext* ctx, GpuBuffer buffer);
void gpu_context_delete_buffer(GpuContext* ctx, GpuBuffer buffer);

// Texture operations
GpuTexture gpu_context_create_texture(GpuContext* ctx, uint32_t width, uint32_t height, const uint8_t* data);
void gpu_context_bind_texture(GpuContext* ctx, GpuTexture texture, uint32_t slot);
void gpu_context_delete_texture(GpuContext* ctx, GpuTexture texture);

// Draw operations
void gpu_context_draw_arrays(GpuContext* ctx, uint32_t mode, int32_t first, int32_t count);
void gpu_context_draw_elements(GpuContext* ctx, uint32_t mode, int32_t count, const uint32_t* indices);

#ifdef __cplusplus
}
#endif

#endif // MOCHI_GPU_CONTEXT_H
