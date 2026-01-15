use std::ffi::CString;
use std::os::raw::{c_char, c_float, c_int, c_uint};
use super::ffi::GpuContextHandle;
use super::effects::{RenderGraph, RenderNode, BlendMode};

extern "C" {
    pub fn gpu_blur_pass(
        ctx: *mut GpuContextHandle,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        radius: c_float,
        samples: c_int,
    );

    pub fn gpu_shadow_pass(
        ctx: *mut GpuContextHandle,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        offset_x: c_float,
        offset_y: c_float,
        color: *const c_float,
        blur: c_float,
        opacity: c_float,
    );

    pub fn gpu_composite_pass(ctx: *mut GpuContextHandle, blend_mode: c_int);

    pub fn gpu_color_adjust_pass(
        ctx: *mut GpuContextHandle,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        brightness: c_float,
        contrast: c_float,
        saturation: c_float,
    );

    pub fn gpu_upload_uniforms(
        ctx: *mut GpuContextHandle,
        name: *const c_char,
        uniform_type: c_int,
        values: *const c_float,
    );

    pub fn gpu_execute_render_graph(
        ctx: *mut GpuContextHandle,
        nodes: *const c_int,
        node_count: c_uint,
        params: *const c_float,
    );
}

/// Execute a render graph on the GPU
pub fn execute_render_graph(ctx: *mut GpuContextHandle, graph: &RenderGraph) {
    if ctx.is_null() {
        return;
    }

    // Encode render graph into arrays for C
    let mut node_types = Vec::new();
    let mut params = Vec::new();

    for node in graph.nodes() {
        match node {
            RenderNode::Clear { color } => {
                node_types.push(0);
                params.extend_from_slice(&[color.x, color.y, color.z, color.w]);
            }
            RenderNode::DrawRect { x, y, width, height, color } => {
                node_types.push(1);
                params.extend_from_slice(&[
                    *x as f32,
                    *y as f32,
                    *width as f32,
                    *height as f32,
                    color.x,
                    color.y,
                    color.z,
                    color.w,
                ]);
            }
            RenderNode::BlurPass { radius, samples } => {
                node_types.push(2);
                params.extend_from_slice(&[*radius, *samples as f32]);
            }
            RenderNode::ShadowPass { offset, color, blur, opacity } => {
                node_types.push(3);
                params.extend_from_slice(&[
                    offset.x,
                    offset.y,
                    color.x,
                    color.y,
                    color.z,
                    color.w,
                    *blur,
                    *opacity,
                ]);
            }
            RenderNode::CompositePass { blend_mode } => {
                node_types.push(4);
                let mode = match blend_mode {
                    BlendMode::Normal => 0,
                    BlendMode::Multiply => 1,
                    BlendMode::Screen => 2,
                    BlendMode::Overlay => 3,
                };
                params.push(mode as f32);
            }
            RenderNode::ColorAdjust { brightness, contrast, saturation } => {
                node_types.push(5);
                params.extend_from_slice(&[*brightness, *contrast, *saturation]);
            }
        }
    }

    unsafe {
        gpu_execute_render_graph(
            ctx,
            node_types.as_ptr(),
            node_types.len() as u32,
            params.as_ptr(),
        );
    }
}
