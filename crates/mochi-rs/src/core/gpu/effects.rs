use glam::{Vec2, Vec4};

/// Effect types that can be applied to UI elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectType {
    Blur,
    Shadow,
    Glow,
    Gradient,
    Brightness,
    Contrast,
    Saturation,
}

/// Individual effect with parameters
#[derive(Debug, Clone)]
pub struct Effect {
    pub effect_type: EffectType,
    pub params: EffectParams,
}

/// Effect parameters (union-like structure)
#[derive(Debug, Clone)]
pub struct EffectParams {
    // Blur parameters
    pub blur_radius: f32,
    pub blur_samples: i32,
    
    // Shadow parameters
    pub shadow_offset: Vec2,
    pub shadow_color: Vec4,
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
    
    // Glow parameters
    pub glow_intensity: f32,
    pub glow_color: Vec4,
    
    // Gradient parameters
    pub gradient_start: Vec4,
    pub gradient_end: Vec4,
    pub gradient_angle: f32,
    
    // Color adjustment parameters
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            blur_radius: 5.0,
            blur_samples: 8,
            shadow_offset: Vec2::new(0.0, 4.0),
            shadow_color: Vec4::new(0.0, 0.0, 0.0, 0.5),
            shadow_blur: 8.0,
            shadow_opacity: 0.5,
            glow_intensity: 1.5,
            glow_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            gradient_start: Vec4::new(1.0, 1.0, 1.0, 1.0),
            gradient_end: Vec4::new(0.0, 0.0, 0.0, 1.0),
            gradient_angle: 0.0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

impl Effect {
    pub fn blur(radius: f32) -> Self {
        Self {
            effect_type: EffectType::Blur,
            params: EffectParams {
                blur_radius: radius,
                ..Default::default()
            },
        }
    }

    pub fn shadow(offset: Vec2, color: Vec4, blur: f32) -> Self {
        Self {
            effect_type: EffectType::Shadow,
            params: EffectParams {
                shadow_offset: offset,
                shadow_color: color,
                shadow_blur: blur,
                ..Default::default()
            },
        }
    }

    pub fn glow(intensity: f32) -> Self {
        Self {
            effect_type: EffectType::Glow,
            params: EffectParams {
                glow_intensity: intensity,
                ..Default::default()
            },
        }
    }

    pub fn gradient(start: Vec4, end: Vec4, angle: f32) -> Self {
        Self {
            effect_type: EffectType::Gradient,
            params: EffectParams {
                gradient_start: start,
                gradient_end: end,
                gradient_angle: angle,
                ..Default::default()
            },
        }
    }

    pub fn brightness(value: f32) -> Self {
        Self {
            effect_type: EffectType::Brightness,
            params: EffectParams {
                brightness: value,
                ..Default::default()
            },
        }
    }

    pub fn contrast(value: f32) -> Self {
        Self {
            effect_type: EffectType::Contrast,
            params: EffectParams {
                contrast: value,
                ..Default::default()
            },
        }
    }

    pub fn saturation(value: f32) -> Self {
        Self {
            effect_type: EffectType::Saturation,
            params: EffectParams {
                saturation: value,
                ..Default::default()
            },
        }
    }
}

/// Stack of effects to be applied in order
#[derive(Debug, Clone, Default)]
pub struct EffectStack {
    effects: Vec<Effect>,
}

impl EffectStack {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn push(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }
}

/// Render graph node representing a rendering operation
#[derive(Debug, Clone)]
pub enum RenderNode {
    /// Clear the target
    Clear { color: Vec4 },
    
    /// Draw a solid rectangle
    DrawRect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Vec4,
    },
    
    /// Apply blur effect
    BlurPass {
        radius: f32,
        samples: i32,
    },
    
    /// Apply shadow effect
    ShadowPass {
        offset: Vec2,
        color: Vec4,
        blur: f32,
        opacity: f32,
    },
    
    /// Composite layers
    CompositePass {
        blend_mode: BlendMode,
    },
    
    /// Apply color adjustment
    ColorAdjust {
        brightness: f32,
        contrast: f32,
        saturation: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

/// Render graph that describes the rendering pipeline
#[derive(Debug, Clone, Default)]
pub struct RenderGraph {
    nodes: Vec<RenderNode>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: RenderNode) {
        self.nodes.push(node);
    }

    pub fn nodes(&self) -> &[RenderNode] {
        &self.nodes
    }

    /// Build a render graph from an effect stack
    pub fn from_effect_stack(
        stack: &EffectStack,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        base_color: Vec4,
    ) -> Self {
        let mut graph = Self::new();

        // Process effects in order
        for effect in stack.effects() {
            match effect.effect_type {
                EffectType::Shadow => {
                    graph.add_node(RenderNode::ShadowPass {
                        offset: effect.params.shadow_offset,
                        color: effect.params.shadow_color,
                        blur: effect.params.shadow_blur,
                        opacity: effect.params.shadow_opacity,
                    });
                }
                EffectType::Blur => {
                    graph.add_node(RenderNode::BlurPass {
                        radius: effect.params.blur_radius,
                        samples: effect.params.blur_samples,
                    });
                }
                EffectType::Glow => {
                    // Glow is implemented as blur + composite
                    graph.add_node(RenderNode::BlurPass {
                        radius: 8.0,
                        samples: 8,
                    });
                    graph.add_node(RenderNode::CompositePass {
                        blend_mode: BlendMode::Screen,
                    });
                }
                EffectType::Brightness | EffectType::Contrast | EffectType::Saturation => {
                    graph.add_node(RenderNode::ColorAdjust {
                        brightness: effect.params.brightness,
                        contrast: effect.params.contrast,
                        saturation: effect.params.saturation,
                    });
                }
                _ => {}
            }
        }

        // Always end with drawing the base shape
        graph.add_node(RenderNode::DrawRect {
            x,
            y,
            width,
            height,
            color: base_color,
        });

        graph
    }
}
