use mochi::{
    card, container, text, Canvas, Color, Element, FragmentShader, GradientShader, NoiseShader,
    RadialGradientShader, RoundedRectShader, ShaderContext, TextRenderer, Vec2, Vec4, WaveShader,
    Window, WindowConfig,
};

// Custom shader: Animated circle pulse
struct PulseShader {
    center: Vec2,
    base_radius: f32,
    color: Vec4,
}

impl FragmentShader for PulseShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let dist = (uv - self.center).length();
        let pulse = (ctx.time * 2.0).sin() * 0.1 + 1.0;
        let radius = self.base_radius * pulse;
        let circle = 1.0 - (dist / radius).min(1.0);
        let alpha = circle.powf(2.0);
        Vec4::new(self.color.x, self.color.y, self.color.z, self.color.w * alpha)
    }
}

// Custom shader: Checkerboard pattern
struct CheckerboardShader {
    color1: Vec4,
    color2: Vec4,
    scale: f32,
}

impl FragmentShader for CheckerboardShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution * self.scale;
        let checker = ((uv.x.floor() + uv.y.floor()) % 2.0).abs();
        if checker < 0.5 { self.color1 } else { self.color2 }
    }
}

// Custom shader: Plasma effect
struct PlasmaShader;

impl FragmentShader for PlasmaShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let time = ctx.time;
        let v1 = (uv.x * 10.0 + time).sin();
        let v2 = ((uv.x + uv.y) * 10.0 + time).sin();
        let v3 = ((uv.x - uv.y) * 10.0 + time).sin();
        let v4 = ((uv.x * uv.x + uv.y * uv.y) * 10.0 + time).sin();
        let plasma = (v1 + v2 + v3 + v4) / 4.0;
        let r = (plasma * 0.5 + 0.5).powf(2.0);
        let g = ((plasma + 0.5) * 0.5 + 0.5).powf(2.0);
        let b = ((plasma + 1.0) * 0.5).powf(2.0);
        Vec4::new(r, g, b, 1.0)
    }
}

// Custom shader: Vignette effect
struct VignetteShader {
    base_color: Vec4,
    intensity: f32,
}

impl FragmentShader for VignetteShader {
    fn fragment(&self, ctx: &ShaderContext) -> Vec4 {
        let uv = ctx.frag_coord / ctx.resolution;
        let center = Vec2::splat(0.5);
        let dist = (uv - center).length();
        let vignette = 1.0 - (dist * self.intensity).min(1.0);
        Vec4::new(
            self.base_color.x * vignette,
            self.base_color.y * vignette,
            self.base_color.z * vignette,
            self.base_color.w,
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load fonts
    let mut text_renderer = TextRenderer::new();
    let inter_regular = include_bytes!("../../../fs/library/shared/fonts/Inter-Regular.ttf");
    text_renderer.load_font("regular", inter_regular)?;
    let inter_bold = include_bytes!("../../../fs/library/shared/fonts/Inter-Bold.ttf");
    text_renderer.load_font("bold", inter_bold)?;

    let config = WindowConfig {
        title: "Mochi - Custom GLSL Shaders".to_string(),
        width: 1400,
        height: 900,
        min_width: Some(800),
        min_height: Some(600),
        decorations: false,
    };

    let mut window = Window::new(config)?;
    let mut time = 0.0f32;

    window.on_draw(move |canvas: &mut Canvas| {
        time += 0.016; // ~60fps

        let width = canvas.width() as i32;
        let height = canvas.height() as i32;

        // RSX-like declarative UI structure
        let margin = 20;
        let card_width = (width - margin * 5) / 4;
        let card_height = 200;
        let spacing = margin;

        // Build UI tree
        let ui = container(0, 0, width, height)
            .background(Color::rgb(15, 15, 20))
            .child(
                // Title bar
                container(0, 0, width, 50)
                    .background(Color::rgb(25, 25, 30))
                    .child(
                        text("Custom GLSL-like Shaders", 20, 20)
                            .size(18.0)
                            .color(Color::rgb(255, 255, 255))
                            .font("bold"),
                    ),
            );

        ui.render(canvas, &text_renderer);

        // Shader grid layout
        let shaders: Vec<(&str, Box<dyn Fn(&mut Canvas, i32, i32, i32, i32, f32)>)> = vec![
            (
                "Linear Gradient",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = GradientShader {
                        color_start: Vec4::new(0.4, 0.2, 0.8, 1.0),
                        color_end: Vec4::new(0.8, 0.2, 0.6, 1.0),
                        angle: 45.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Radial Gradient",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = RadialGradientShader {
                        color_center: Vec4::new(1.0, 0.8, 0.2, 1.0),
                        color_edge: Vec4::new(0.8, 0.2, 0.2, 1.0),
                        center: Vec2::splat(0.5),
                        radius: 0.7,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Rounded SDF",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = RoundedRectShader {
                        color: Vec4::new(0.2, 0.6, 1.0, 1.0),
                        rect_pos: Vec2::new(10.0, 10.0),
                        rect_size: Vec2::new((w - 20) as f32, (h - 20) as f32),
                        corner_radius: 20.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Noise",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = NoiseShader {
                        base_color: Vec4::new(0.3, 0.8, 0.5, 1.0),
                        scale: 10.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Pulse (Animated)",
                Box::new(|canvas, x, y, w, h, time| {
                    canvas.fill_rect(x, y, w, h, Color::rgb(20, 20, 25));
                    let mut ctx = ShaderContext::new(Vec2::new(w as f32, h as f32));
                    ctx.time = time;
                    let shader = PulseShader {
                        center: Vec2::splat(0.5),
                        base_radius: 0.3,
                        color: Vec4::new(1.0, 0.3, 0.5, 1.0),
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Checkerboard",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = CheckerboardShader {
                        color1: Vec4::new(0.9, 0.9, 0.9, 1.0),
                        color2: Vec4::new(0.3, 0.3, 0.3, 1.0),
                        scale: 8.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Plasma (Animated)",
                Box::new(|canvas, x, y, w, h, time| {
                    let mut ctx = ShaderContext::new(Vec2::new(w as f32, h as f32));
                    ctx.time = time;
                    canvas.execute_shader(x, y, w, h, &PlasmaShader);
                }),
            ),
            (
                "Vignette",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = VignetteShader {
                        base_color: Vec4::new(0.5, 0.3, 0.8, 1.0),
                        intensity: 1.5,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Wave",
                Box::new(|canvas, x, y, w, h, time| {
                    let mut ctx = ShaderContext::new(Vec2::new(w as f32, h as f32));
                    ctx.time = time;
                    let shader = WaveShader {
                        base_color: Vec4::new(0.2, 0.5, 0.9, 1.0),
                        amplitude: 0.05,
                        frequency: 10.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Vertical Gradient",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = GradientShader {
                        color_start: Vec4::new(0.1, 0.9, 0.5, 1.0),
                        color_end: Vec4::new(0.1, 0.3, 0.9, 1.0),
                        angle: 90.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Diagonal Gradient",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = GradientShader {
                        color_start: Vec4::new(1.0, 0.5, 0.0, 1.0),
                        color_end: Vec4::new(1.0, 0.0, 0.5, 1.0),
                        angle: 135.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
            (
                "Horizontal Gradient",
                Box::new(|canvas, x, y, w, h, _time| {
                    let shader = GradientShader {
                        color_start: Vec4::new(0.0, 0.8, 0.8, 1.0),
                        color_end: Vec4::new(0.8, 0.0, 0.8, 1.0),
                        angle: 180.0,
                    };
                    canvas.execute_shader(x, y, w, h, &shader);
                }),
            ),
        ];

        // Render shader grid
        for (i, (label, render_fn)) in shaders.iter().enumerate() {
            let row = i / 4;
            let col = i % 4;
            let x = margin + col as i32 * (card_width + spacing);
            let y = 70 + row as i32 * (card_height + spacing);

            // Render shader
            render_fn(canvas, x, y, card_width, card_height, time);

            // Render label
            text_renderer.render(
                canvas,
                label,
                x + 10,
                y + card_height - 30,
                14.0,
                Color::rgb(255, 255, 255),
                "bold",
            );
        }
    });

    window.run()
}
