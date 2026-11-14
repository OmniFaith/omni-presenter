use crate::core::models::{PresentationTheme, theme_options::ShaderOptions};
use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
mod custom;

pub const PRESETS: [(&str, &str, &str); 7] = [
    ("aurora", "settings-aurora", "settings-moving-curtains-of-light"),
    ("waves", "settings-waves", "settings-smooth-flowing-layers"),
    ("silk", "settings-silk", "settings-delicate-folds-and-reflections"),
    ("particles", "settings-particles", "settings-floating-points-of-light"),
    ("rings", "settings-rings", "settings-concentric-ripples"),
    ("fluid", "settings-fluid", "settings-ink-and-organic-motion"),
    ("bokeh", "settings-bokeh", "settings-soft-atmospheric-lights"),
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    values: [[f32; 4]; 7],
}
struct Instance {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct InstanceKey {
    viewport: egui::ViewportId,
    slot: usize,
}
#[derive(Clone, Default)]
struct CallbackSlots {
    pass: u64,
    next: usize,
}
struct Resources {
    pipeline: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
    custom: HashMap<u64, Option<wgpu::RenderPipeline>>,
    layout: wgpu::BindGroupLayout,
    instances: HashMap<InstanceKey, Instance>,
}
struct ShaderCallback {
    key: InstanceKey,
    uniforms: Uniforms,
    shader: Option<crate::core::plugins::PluginShader>,
    shader_key: u64,
}

pub fn install(state: &egui_wgpu::RenderState, ctx: &egui::Context) {
    let device = &state.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("omni presenter procedural backgrounds"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../assets/shaders/background.wgsl").into(),
        ),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("background parameters"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("background pipeline layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("omni presenter shader backgrounds"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: state.target_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    state.renderer.write().callback_resources.insert(Resources {
        pipeline,
        format: state.target_format,
        custom: HashMap::new(),
        layout,
        instances: HashMap::new(),
    });
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("gpu_backgrounds_ready"), true));
}

pub fn kind_index(kind: &str) -> usize {
    PRESETS.iter().position(|p| p.0 == kind).unwrap_or(0)
}

pub fn clock() -> f32 {
    static START: std::sync::LazyLock<std::time::Instant> =
        std::sync::LazyLock::new(std::time::Instant::now);
    START.elapsed().as_secs_f32()
}

fn uniforms(size: egui::Vec2, time: f32, radius: f32, shader: &ShaderOptions) -> Uniforms {
    let color = |index: usize| {
        let c = shader.colors[index];
        [
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
            1.0,
        ]
    };
    Uniforms {
        values: [
            [size.x, size.y, time, kind_index(&shader.kind) as f32],
            [
                shader.speed,
                shader.scale.max(0.05),
                shader.intensity,
                shader.softness,
            ],
            [
                shader.detail,
                shader.direction.to_radians(),
                shader.seed,
                shader.vignette,
            ],
            color(0),
            color(1),
            color(2),
            [shader.grain, radius, 1.0, 0.0],
        ],
    }
}

pub fn paint(
    painter: &egui::Painter,
    rect: egui::Rect,
    theme: &PresentationTheme,
    radius: f32,
) -> bool {
    if !painter
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(egui::Id::unique("gpu_backgrounds_ready")))
        .unwrap_or(false)
    {
        return false;
    }
    if !rect.is_positive() || !painter.clip_rect().intersects(rect) {
        return true;
    }
    let mut callback = shader_callback(painter.ctx(), rect, theme, radius);
    callback.uniforms.values[6][2] = painter.opacity();
    painter.add(egui_wgpu::Callback::new_paint_callback(rect, callback));
    if theme.options.shader.speed > 0.0 {
        painter
            .ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
    true
}

fn shader_callback(
    ctx: &egui::Context,
    rect: egui::Rect,
    theme: &PresentationTheme,
    radius: f32,
) -> ShaderCallback {
    let viewport = ctx.viewport_id();
    let pass = ctx.cumulative_pass_nr();
    // Each draw in a pass needs its own uniform buffer, including overlapping previews of the
    // same theme. Reuse these slots next pass rather than caching every scroll position.
    let key = ctx.data_mut(|data| {
        let slots = data.get_temp_mut_or_default::<CallbackSlots>(
            egui::Id::unique("shader_callback_slots").with(viewport),
        );
        if slots.pass != pass {
            slots.pass = pass;
            slots.next = 0;
        }
        let key = InstanceKey {
            viewport,
            slot: slots.next,
        };
        slots.next += 1;
        key
    });
    let shader = theme.options.shader.custom.clone();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Some(shader) = &shader { shader.wgsl.hash(&mut hasher); }
    ShaderCallback {
        key,
        shader,
        shader_key: hasher.finish(),
        uniforms: uniforms(rect.size(), clock(), radius, &theme.options.shader),
    }
}

impl egui_wgpu::CallbackTrait for ShaderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        _: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(resources) = resources.get_mut::<Resources>() else {
            return Vec::new();
        };
        if let Some(shader) = &self.shader
            && !resources.custom.contains_key(&self.shader_key) {
                let pipeline = custom::pipeline(device, resources.format, &resources.layout, shader);
                resources.custom.insert(self.shader_key, pipeline);
            }
        // Never evict buffers here: callbacks prepared earlier still need them during paint.
        let instance = resources.instances.entry(self.key).or_insert_with(|| {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("background uniform buffer"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("background parameters"),
                layout: &resources.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            Instance { buffer, bind_group }
        });
        queue.write_buffer(&instance.buffer, 0, bytemuck::bytes_of(&self.uniforms));
        Vec::new()
    }

    fn paint(
        &self,
        _: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        if let Some(resources) = resources.get::<Resources>()
            && let Some(instance) = resources.instances.get(&self.key) {
                let pipeline = resources.custom.get(&self.shader_key).and_then(Option::as_ref).unwrap_or(&resources.pipeline);
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &instance.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
    }
}

pub fn default_themes() -> Vec<PresentationTheme> {
    PRESETS
        .iter()
        .enumerate()
        .map(|(index, (kind, name, _))| {
            let mut theme = PresentationTheme::default();
            theme.id = format!("shader_{kind}");
            let translated = crate::core::i18n::Language::English.text(name);
            theme.name = crate::core::i18n::Language::English
                .format("services-animated", &[("name", translated)]);
            theme.bg_type = "shader".into();
            theme.transition = "blur".into();
            theme.font_size = 46.0;
            theme.font_weight = 700;
            theme.line_spacing = 1.36;
            theme.letter_spacing = 0.6;
            theme.shadow_enabled = true;
            theme.shadow_blur = 10.0;
            theme.shadow_offset_x = 0.0;
            theme.shadow_offset_y = 3.0;
            theme.shadow_color = egui::Color32::from_black_alpha(230);
            theme.options.shader.kind = (*kind).into();
            theme.options.shader.vignette = 0.40;
            theme.options.shader.grain = 0.012;
            let (speed, colors) = match *kind {
                "aurora" => (0.22, [[6, 12, 28], [24, 160, 130], [90, 60, 180]]),
                "nebula" => (0.18, [[8, 10, 26], [110, 36, 132], [54, 118, 208]]),
                "waves" => (0.20, [[5, 18, 38], [18, 88, 148], [42, 180, 200]]),
                "silk" => (0.18, [[24, 14, 12], [138, 76, 60], [220, 175, 130]]),
                "particles" => (0.24, [[8, 9, 14], [50, 45, 30], [234, 195, 115]]),
                "rings" => (0.20, [[12, 8, 26], [70, 42, 120], [140, 95, 210]]),
                "fluid" => (0.20, [[6, 16, 24], [20, 95, 110], [70, 190, 175]]),
                "grid" => (0.22, [[10, 14, 22], [35, 65, 115], [90, 145, 230]]),
                "bokeh" => (0.20, [[18, 12, 10], [95, 52, 28], [240, 170, 90]]),
                "prism" => (0.18, [[10, 10, 24], [75, 45, 130], [130, 200, 215]]),
                _ => (0.25, theme.options.shader.colors),
            };
            theme.options.shader.speed = speed;
            theme.options.shader.colors = colors;
            theme.sort_order = 10 + index as i32;
            theme
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_and_resizing_reuse_shader_buffers_without_aliasing_draws() {
        let ctx = egui::Context::default();
        let theme = default_themes().remove(0);
        let mut allocated = std::collections::HashSet::new();
        let stage = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(640.0, 360.0));
        for frame in 0..300 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_ui| {
                let live = shader_callback(&ctx, stage, &theme, 10.0);
                let preview = shader_callback(&ctx, stage, &theme, 10.0);
                let mut draws = std::collections::HashSet::from([live.key, preview.key]);
                assert_eq!(
                    draws.len(),
                    2,
                    "even identical rectangles need distinct uniforms"
                );
                for card in 0..10 {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(700.0, card as f32 * 100.0 - frame as f32 * 3.0),
                        egui::vec2(180.0 + (frame % 20) as f32, 100.0),
                    );
                    let callback = shader_callback(&ctx, rect, &theme, 10.0);
                    assert!(draws.insert(callback.key));
                    assert_eq!(callback.uniforms.values[0][0], rect.width());
                    assert_eq!(callback.uniforms.values[0][1], rect.height());
                }
                allocated.extend(draws);
                assert_eq!(
                    allocated.len(),
                    12,
                    "scrolling must not allocate new buffer slots"
                );
            });
            output.textures_delta.clear();
        }
    }

    #[test]
    fn shader_slots_are_independent_for_each_viewport() {
        let ctx = egui::Context::default();
        let theme = default_themes().remove(0);
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(160.0, 90.0));
        let secondary = egui::ViewportId::from_hash_of("shader_test_output");
        let mut keys = Vec::new();
        for viewport_id in [egui::ViewportId::ROOT, secondary, egui::ViewportId::ROOT] {
            let mut input = egui::RawInput {
                viewport_id,
                ..Default::default()
            };
            input.viewports.entry(viewport_id).or_default();
            let mut output = ctx.run_ui(input, |_ui| {
                keys.push(shader_callback(&ctx, rect, &theme, 0.0).key);
            });
            output.textures_delta.clear();
        }
        assert_ne!(
            keys[0], keys[1],
            "operator and output cannot share uniforms"
        );
        assert_eq!(
            keys[0], keys[2],
            "each viewport reuses its own slots next pass"
        );
    }

    #[test]
    fn fully_clipped_shader_cards_do_not_allocate_draw_slots() {
        let ctx = egui::Context::default();
        ctx.data_mut(|data| data.insert_temp(egui::Id::unique("gpu_backgrounds_ready"), true));
        let theme = default_themes().remove(0);
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(160.0, 90.0));
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter().with_clip_rect(rect);
            assert!(paint(&painter, rect, &theme, 10.0));
            for offset in 1..300 {
                assert!(paint(
                    &painter,
                    rect.translate(egui::vec2(0.0, offset as f32 * 100.0)),
                    &theme,
                    10.0
                ));
            }
            assert_eq!(shader_callback(&ctx, rect, &theme, 10.0).key.slot, 1);
        });
        assert_eq!(
            output
                .shapes
                .iter()
                .filter(|shape| matches!(shape.shape, egui::Shape::Callback(_)))
                .count(),
            1
        );
        output.textures_delta.clear();
    }

    #[test]
    fn all_background_shader_branches_validate() {
        let module =
            naga::front::wgsl::parse_str(include_str!("../../assets/shaders/background.wgsl"))
                .unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        assert_eq!(super::default_themes().len(), 7);
    }
}
