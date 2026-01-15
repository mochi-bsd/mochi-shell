#include "passes.h"
#include <string.h>
#include <math.h>

// Platform-specific includes
#if defined(__linux__) || defined(__FreeBSD__)
    #include <GLES3/gl3.h>
    #define HAS_OPENGL 1
#elif defined(_WIN32)
    #include <GL/gl.h>
    #define HAS_OPENGL 1
#elif defined(__APPLE__)
    #include <OpenGL/gl3.h>
    #define HAS_OPENGL 1
#else
    #define HAS_OPENGL 0
#endif

void gpu_blur_pass(
    GpuContext* ctx,
    int32_t x, int32_t y,
    uint32_t width, uint32_t height,
    float radius,
    int32_t samples
) {
    if (!ctx) return;
    
    // TODO: Implement optimized separable blur
    // 1. Horizontal blur pass
    // 2. Vertical blur pass
    // This is much faster than full 2D blur
}

void gpu_shadow_pass(
    GpuContext* ctx,
    int32_t x, int32_t y,
    uint32_t width, uint32_t height,
    float offset_x, float offset_y,
    float color[4],
    float blur,
    float opacity
) {
    if (!ctx || !color) return;
    
    // TODO: Implement shadow rendering
    // 1. Render shape to alpha texture
    // 2. Blur alpha texture
    // 3. Multiply by shadow color
    // 4. Composite with offset
}


void gpu_composite_pass(
    GpuContext* ctx,
    int32_t blend_mode
) {
    if (!ctx) return;
    
    #if HAS_OPENGL
    // Set OpenGL blend mode
    glEnable(GL_BLEND);
    switch (blend_mode) {
        case 0: // Normal
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
            break;
        case 1: // Multiply
            glBlendFunc(GL_DST_COLOR, GL_ZERO);
            break;
        case 2: // Screen
            glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_COLOR);
            break;
        case 3: // Overlay
            glBlendFunc(GL_SRC_ALPHA, GL_ONE);
            break;
    }
    #else
    (void)blend_mode; // Suppress unused parameter warning
    #endif
}

void gpu_color_adjust_pass(
    GpuContext* ctx,
    int32_t x, int32_t y,
    uint32_t width, uint32_t height,
    float brightness,
    float contrast,
    float saturation
) {
    if (!ctx) return;
    
    // TODO: Implement color adjustment shader
    // Use fragment shader to adjust colors in real-time
}

void gpu_upload_uniforms(
    GpuContext* ctx,
    const char* name,
    int32_t type,
    const float* values
) {
    if (!ctx || !name || !values) return;
    
    switch (type) {
        case 0: // float
            gpu_context_set_uniform_float(ctx, name, values[0]);
            break;
        case 1: // vec2
            gpu_context_set_uniform_vec2(ctx, name, values[0], values[1]);
            break;
        case 2: // vec3
            gpu_context_set_uniform_vec3(ctx, name, values[0], values[1], values[2]);
            break;
        case 3: // vec4
            gpu_context_set_uniform_vec4(ctx, name, values[0], values[1], values[2], values[3]);
            break;
    }
}

void gpu_execute_render_graph(
    GpuContext* ctx,
    const int32_t* nodes,
    uint32_t node_count,
    const float* params
) {
    if (!ctx || !nodes || !params) return;
    
    // Execute each node in the render graph
    uint32_t param_offset = 0;
    
    for (uint32_t i = 0; i < node_count; i++) {
        int32_t node_type = nodes[i];
        
        switch (node_type) {
            case 0: // Clear
                gpu_context_clear(ctx, params[param_offset], params[param_offset+1],
                                 params[param_offset+2], params[param_offset+3]);
                param_offset += 4;
                break;
                
            case 1: // DrawRect
                // TODO: Implement rect drawing
                param_offset += 8; // x, y, w, h, r, g, b, a
                break;
                
            case 2: // BlurPass
                gpu_blur_pass(ctx, 0, 0, 0, 0, params[param_offset], (int32_t)params[param_offset+1]);
                param_offset += 2;
                break;
                
            case 3: // ShadowPass
                {
                    float color[4] = {params[param_offset+2], params[param_offset+3],
                                     params[param_offset+4], params[param_offset+5]};
                    gpu_shadow_pass(ctx, 0, 0, 0, 0,
                                   params[param_offset], params[param_offset+1],
                                   color, params[param_offset+6], params[param_offset+7]);
                    param_offset += 8;
                }
                break;
                
            case 4: // CompositePass
                gpu_composite_pass(ctx, (int32_t)params[param_offset]);
                param_offset += 1;
                break;
                
            case 5: // ColorAdjust
                gpu_color_adjust_pass(ctx, 0, 0, 0, 0,
                                     params[param_offset], params[param_offset+1], params[param_offset+2]);
                param_offset += 3;
                break;
        }
    }
}
