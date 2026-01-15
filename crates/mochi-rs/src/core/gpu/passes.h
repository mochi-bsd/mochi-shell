#ifndef MOCHI_GPU_PASSES_H
#define MOCHI_GPU_PASSES_H

#include <stdint.h>
#include "gpucontext.h"

#ifdef __cplusplus
extern "C" {
#endif

// Rendering pass functions - optimized GPU kernels

/// Apply box blur to a texture region
/// @param ctx GPU context
/// @param x, y, width, height Region to blur
/// @param radius Blur radius in pixels
/// @param samples Number of samples (quality vs performance)
void gpu_blur_pass(
    GpuContext* ctx,
    int32_t x, int32_t y,
    uint32_t width, uint32_t height,
    float radius,
    int32_t samples
);

/// Apply shadow effect
/// @param ctx GPU context
/// @param x, y, width, height Shape bounds
/// @param offset_x, offset_y Shadow offset
/// @param color Shadow color (RGBA, 0-1 range)
/// @param blur Shadow blur radius
/// @param opacity Shadow opacity (0-1)
void gpu_shadow_pass(
    GpuContext* ctx,
    int32_t x, int32_t y,
    uint32_t width, uint32_t height,
    float offset_x, float offset_y,
    float color[4],
    float blur,
    float opacity
);

/// Composite two layers together
/// @param ctx GPU context
/// @param blend_mode 0=Normal, 1=Multiply, 2=Screen, 3=Overlay
void gpu_composite_pass(
    GpuContext* ctx,
    int32_t blend_mode
);

/// Apply color adjustments (brightness, contrast, saturation)
/// @param ctx GPU context
/// @param x, y, width, height Region to adjust
/// @param brightness Brightness multiplier (1.0 = no change)
/// @param contrast Contrast multiplier (1.0 = no change)
/// @param saturation Saturation multiplier (1.0 = no change)
void gpu_color_adjust_pass(
    GpuContext* ctx,
    int32_t x, int32_t y,
    uint32_t width, uint32_t height,
    float brightness,
    float contrast,
    float saturation
);

/// Upload uniform values to current shader
/// @param ctx GPU context
/// @param name Uniform name
/// @param type 0=float, 1=vec2, 2=vec3, 3=vec4
/// @param values Array of float values
void gpu_upload_uniforms(
    GpuContext* ctx,
    const char* name,
    int32_t type,
    const float* values
);

/// Execute a complete render graph
/// @param ctx GPU context
/// @param nodes Array of render nodes (encoded as integers)
/// @param node_count Number of nodes
/// @param params Array of parameters for each node
void gpu_execute_render_graph(
    GpuContext* ctx,
    const int32_t* nodes,
    uint32_t node_count,
    const float* params
);

#ifdef __cplusplus
}
#endif

#endif // MOCHI_GPU_PASSES_H
