#include "gpucontext.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Platform-specific includes
#ifdef __linux__
    #include <EGL/egl.h>
    #include <GLES3/gl3.h>
    #define USE_EGL 1
    #define HAS_OPENGL 1
#elif defined(__FreeBSD__)
    // FreeBSD: Use GLES3 from /usr/local/include
    #include <EGL/egl.h>
    #include <GLES3/gl3.h>
    #define USE_EGL 1
    #define HAS_OPENGL 1
#elif defined(_WIN32)
    #include <windows.h>
    #include <GL/gl.h>
    #define USE_WGL 1
    #define HAS_OPENGL 1
#elif defined(__APPLE__)
    #include <OpenGL/gl3.h>
    #define USE_CGL 1
    #define HAS_OPENGL 1
#else
    // No OpenGL support - define types for compilation
    typedef unsigned int GLuint;
    typedef int GLint;
    typedef unsigned int GLenum;
    #define HAS_OPENGL 0
#endif

// Internal GPU context structure
struct GpuContext {
    GpuBackendType backend_type;
    uint32_t width;
    uint32_t height;
    
    // OpenGL/GLES state
    #ifdef USE_EGL
    EGLDisplay egl_display;
    EGLContext egl_context;
    EGLSurface egl_surface;
    EGLConfig egl_config;
    #endif
    
    // Current shader program
    GLuint current_program;
    
    // Device info
    char device_name[256];
    char vendor_name[128];
    char driver_version[64];
};

// Helper: Try to initialize OpenGL ES
static bool try_init_opengles(GpuContext* ctx) {
    #ifdef USE_EGL
    ctx->egl_display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    if (ctx->egl_display == EGL_NO_DISPLAY) {
        return false;
    }
    
    EGLint major, minor;
    if (!eglInitialize(ctx->egl_display, &major, &minor)) {
        return false;
    }
    
    // Choose config for GLES 3.0
    EGLint config_attribs[] = {
        EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
        EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
        EGL_RED_SIZE, 8,
        EGL_GREEN_SIZE, 8,
        EGL_BLUE_SIZE, 8,
        EGL_ALPHA_SIZE, 8,
        EGL_DEPTH_SIZE, 24,
        EGL_NONE
    };
    
    EGLint num_configs;
    if (!eglChooseConfig(ctx->egl_display, config_attribs, &ctx->egl_config, 1, &num_configs)) {
        eglTerminate(ctx->egl_display);
        return false;
    }
    
    // Create pbuffer surface for offscreen rendering
    EGLint pbuffer_attribs[] = {
        EGL_WIDTH, (EGLint)ctx->width,
        EGL_HEIGHT, (EGLint)ctx->height,
        EGL_NONE
    };
    
    ctx->egl_surface = eglCreatePbufferSurface(ctx->egl_display, ctx->egl_config, pbuffer_attribs);
    if (ctx->egl_surface == EGL_NO_SURFACE) {
        eglTerminate(ctx->egl_display);
        return false;
    }
    
    // Create context
    EGLint context_attribs[] = {
        EGL_CONTEXT_CLIENT_VERSION, 3,
        EGL_NONE
    };
    
    ctx->egl_context = eglCreateContext(ctx->egl_display, ctx->egl_config, EGL_NO_CONTEXT, context_attribs);
    if (ctx->egl_context == EGL_NO_CONTEXT) {
        eglDestroySurface(ctx->egl_display, ctx->egl_surface);
        eglTerminate(ctx->egl_display);
        return false;
    }
    
    // Make context current
    if (!eglMakeCurrent(ctx->egl_display, ctx->egl_surface, ctx->egl_surface, ctx->egl_context)) {
        eglDestroyContext(ctx->egl_display, ctx->egl_context);
        eglDestroySurface(ctx->egl_display, ctx->egl_surface);
        eglTerminate(ctx->egl_display);
        return false;
    }
    
    // Get device info
    #ifdef HAS_OPENGL
    const char* vendor = (const char*)glGetString(GL_VENDOR);
    const char* renderer = (const char*)glGetString(GL_RENDERER);
    const char* version = (const char*)glGetString(GL_VERSION);
    
    if (vendor) strncpy(ctx->vendor_name, vendor, sizeof(ctx->vendor_name) - 1);
    if (renderer) strncpy(ctx->device_name, renderer, sizeof(ctx->device_name) - 1);
    if (version) strncpy(ctx->driver_version, version, sizeof(ctx->driver_version) - 1);
    #endif
    
    ctx->backend_type = GPU_BACKEND_OPENGLES;
    return true;
    #else
    (void)ctx; // Suppress unused parameter warning
    return false;
    #endif
}

// Helper: Try to initialize OpenGL (for FreeBSD and other platforms without EGL)
static bool try_init_opengl(GpuContext* ctx) {
    #if defined(__FreeBSD__) || defined(__APPLE__)
    // On FreeBSD/macOS, we can't easily create an OpenGL context without a window
    // For now, just report that OpenGL is available but not initialized
    // In a real implementation, this would use platform-specific context creation
    
    strncpy(ctx->vendor_name, "System", sizeof(ctx->vendor_name) - 1);
    strncpy(ctx->device_name, "OpenGL (not initialized)", sizeof(ctx->device_name) - 1);
    strncpy(ctx->driver_version, "N/A", sizeof(ctx->driver_version) - 1);
    
    ctx->backend_type = GPU_BACKEND_OPENGL;
    return false; // Return false since we can't actually use it yet
    #else
    (void)ctx; // Suppress unused parameter warning
    return false;
    #endif
}

// Helper: Try to initialize Vulkan
static bool try_init_vulkan(GpuContext* ctx) {
    // TODO: Implement Vulkan initialization
    // This would use the Vulkan API directly
    return false;
}

// Public API implementation

GpuContext* gpu_context_create(uint32_t width, uint32_t height) {
    GpuContext* ctx = (GpuContext*)calloc(1, sizeof(GpuContext));
    if (!ctx) {
        return NULL;
    }
    
    ctx->width = width;
    ctx->height = height;
    ctx->backend_type = GPU_BACKEND_NONE;
    
    // Try backends in priority order: Vulkan > OpenGL > OpenGL ES
    if (try_init_vulkan(ctx)) {
        printf("GPU: Initialized Vulkan backend\n");
        return ctx;
    }
    
    if (try_init_opengl(ctx)) {
        printf("GPU: Initialized OpenGL backend\n");
        return ctx;
    }
    
    if (try_init_opengles(ctx)) {
        printf("GPU: Initialized OpenGL ES backend\n");
        return ctx;
    }
    
    // No GPU backend available
    printf("GPU: No hardware backend available, using CPU fallback\n");
    free(ctx);
    return NULL;
}

void gpu_context_destroy(GpuContext* ctx) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        eglMakeCurrent(ctx->egl_display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
        eglDestroyContext(ctx->egl_display, ctx->egl_context);
        eglDestroySurface(ctx->egl_display, ctx->egl_surface);
        eglTerminate(ctx->egl_display);
    }
    #endif
    
    free(ctx);
}

bool gpu_context_is_valid(const GpuContext* ctx) {
    return ctx != NULL && ctx->backend_type != GPU_BACKEND_NONE;
}

GpuBackendType gpu_context_get_backend(const GpuContext* ctx) {
    return ctx ? ctx->backend_type : GPU_BACKEND_NONE;
}

void gpu_context_get_device_info(const GpuContext* ctx, GpuDeviceInfo* info) {
    if (!ctx || !info) return;
    
    memset(info, 0, sizeof(GpuDeviceInfo));
    info->backend_type = ctx->backend_type;
    strncpy(info->device_name, ctx->device_name, sizeof(info->device_name) - 1);
    strncpy(info->vendor_name, ctx->vendor_name, sizeof(info->vendor_name) - 1);
    strncpy(info->driver_version, ctx->driver_version, sizeof(info->driver_version) - 1);
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLint max_texture_size;
        glGetIntegerv(GL_MAX_TEXTURE_SIZE, &max_texture_size);
        info->max_texture_size = (uint32_t)max_texture_size;
        info->supports_compute = false; // GLES 3.0 doesn't have compute shaders
    }
    #endif
}

void gpu_context_clear(GpuContext* ctx, float r, float g, float b, float a) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glClearColor(r, g, b, a);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }
    #endif
}

void gpu_context_viewport(GpuContext* ctx, int32_t x, int32_t y, uint32_t width, uint32_t height) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glViewport(x, y, width, height);
        ctx->width = width;
        ctx->height = height;
    }
    #endif
}

void gpu_context_present(GpuContext* ctx) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        eglSwapBuffers(ctx->egl_display, ctx->egl_surface);
    }
    #endif
}

GpuShader gpu_context_create_shader(GpuContext* ctx, const char* vertex_src, const char* fragment_src) {
    if (!ctx || !vertex_src || !fragment_src) return 0;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        // Compile vertex shader
        GLuint vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        glShaderSource(vertex_shader, 1, &vertex_src, NULL);
        glCompileShader(vertex_shader);
        
        GLint success;
        glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &success);
        if (!success) {
            char info_log[512];
            glGetShaderInfoLog(vertex_shader, 512, NULL, info_log);
            printf("Vertex shader compilation failed: %s\n", info_log);
            glDeleteShader(vertex_shader);
            return 0;
        }
        
        // Compile fragment shader
        GLuint fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        glShaderSource(fragment_shader, 1, &fragment_src, NULL);
        glCompileShader(fragment_shader);
        
        glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &success);
        if (!success) {
            char info_log[512];
            glGetShaderInfoLog(fragment_shader, 512, NULL, info_log);
            printf("Fragment shader compilation failed: %s\n", info_log);
            glDeleteShader(vertex_shader);
            glDeleteShader(fragment_shader);
            return 0;
        }
        
        // Link program
        GLuint program = glCreateProgram();
        glAttachShader(program, vertex_shader);
        glAttachShader(program, fragment_shader);
        glLinkProgram(program);
        
        glGetProgramiv(program, GL_LINK_STATUS, &success);
        if (!success) {
            char info_log[512];
            glGetProgramInfoLog(program, 512, NULL, info_log);
            printf("Shader program linking failed: %s\n", info_log);
            glDeleteShader(vertex_shader);
            glDeleteShader(fragment_shader);
            glDeleteProgram(program);
            return 0;
        }
        
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);
        
        return (GpuShader)program;
    }
    #endif
    
    return 0;
}

void gpu_context_use_shader(GpuContext* ctx, GpuShader shader) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glUseProgram((GLuint)shader);
        ctx->current_program = (GLuint)shader;
    }
    #endif
}

void gpu_context_delete_shader(GpuContext* ctx, GpuShader shader) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glDeleteProgram((GLuint)shader);
    }
    #endif
}

void gpu_context_set_uniform_float(GpuContext* ctx, const char* name, float value) {
    if (!ctx || !name) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLint location = glGetUniformLocation(ctx->current_program, name);
        if (location >= 0) {
            glUniform1f(location, value);
        }
    }
    #endif
}

void gpu_context_set_uniform_vec2(GpuContext* ctx, const char* name, float x, float y) {
    if (!ctx || !name) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLint location = glGetUniformLocation(ctx->current_program, name);
        if (location >= 0) {
            glUniform2f(location, x, y);
        }
    }
    #endif
}

void gpu_context_set_uniform_vec3(GpuContext* ctx, const char* name, float x, float y, float z) {
    if (!ctx || !name) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLint location = glGetUniformLocation(ctx->current_program, name);
        if (location >= 0) {
            glUniform3f(location, x, y, z);
        }
    }
    #endif
}

void gpu_context_set_uniform_vec4(GpuContext* ctx, const char* name, float x, float y, float z, float w) {
    if (!ctx || !name) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLint location = glGetUniformLocation(ctx->current_program, name);
        if (location >= 0) {
            glUniform4f(location, x, y, z, w);
        }
    }
    #endif
}

void gpu_context_set_uniform_mat4(GpuContext* ctx, const char* name, const float* matrix) {
    if (!ctx || !name || !matrix) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLint location = glGetUniformLocation(ctx->current_program, name);
        if (location >= 0) {
            glUniformMatrix4fv(location, 1, GL_FALSE, matrix);
        }
    }
    #endif
}

GpuBuffer gpu_context_create_buffer(GpuContext* ctx, const float* data, uint32_t size) {
    if (!ctx || !data) return 0;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLuint buffer;
        glGenBuffers(1, &buffer);
        glBindBuffer(GL_ARRAY_BUFFER, buffer);
        glBufferData(GL_ARRAY_BUFFER, size, data, GL_STATIC_DRAW);
        return (GpuBuffer)buffer;
    }
    #endif
    
    return 0;
}

void gpu_context_bind_buffer(GpuContext* ctx, GpuBuffer buffer) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glBindBuffer(GL_ARRAY_BUFFER, (GLuint)buffer);
    }
    #endif
}

void gpu_context_delete_buffer(GpuContext* ctx, GpuBuffer buffer) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLuint buf = (GLuint)buffer;
        glDeleteBuffers(1, &buf);
    }
    #endif
}

GpuTexture gpu_context_create_texture(GpuContext* ctx, uint32_t width, uint32_t height, const uint8_t* data) {
    if (!ctx) return 0;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLuint texture;
        glGenTextures(1, &texture);
        glBindTexture(GL_TEXTURE_2D, texture);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, data);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        return (GpuTexture)texture;
    }
    #endif
    
    return 0;
}

void gpu_context_bind_texture(GpuContext* ctx, GpuTexture texture, uint32_t slot) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glActiveTexture(GL_TEXTURE0 + slot);
        glBindTexture(GL_TEXTURE_2D, (GLuint)texture);
    }
    #endif
}

void gpu_context_delete_texture(GpuContext* ctx, GpuTexture texture) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        GLuint tex = (GLuint)texture;
        glDeleteTextures(1, &tex);
    }
    #endif
}

void gpu_context_draw_arrays(GpuContext* ctx, uint32_t mode, int32_t first, int32_t count) {
    if (!ctx) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glDrawArrays(mode, first, count);
    }
    #endif
}

void gpu_context_draw_elements(GpuContext* ctx, uint32_t mode, int32_t count, const uint32_t* indices) {
    if (!ctx || !indices) return;
    
    #ifdef USE_EGL
    if (ctx->backend_type == GPU_BACKEND_OPENGLES) {
        glDrawElements(mode, count, GL_UNSIGNED_INT, indices);
    }
    #endif
}
