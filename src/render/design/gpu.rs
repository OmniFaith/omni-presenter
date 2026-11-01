use super::sprites::{self, Sprite};
use crate::core::design::{BlendMode, DesignDocument};
use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    values: [[f32; 4]; 10],
}
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct Key {
    viewport: egui::ViewportId,
    slot: usize,
}
#[derive(Clone, Default)]
struct Slots {
    pass: u64,
    next: usize,
}
struct Resources {
    layer: wgpu::RenderPipeline,
    display: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    images: HashMap<u64, (wgpu::TextureView, u64)>,
    instances: HashMap<Key, Instance>,
}
struct Instance {
    views: [wgpu::TextureView; 2],
    size: [u32; 2],
    groups: Vec<wgpu::BindGroup>,
    display: wgpu::BindGroup,
}
struct Callback {
    key: Key,
    frame: u64,
    size: [u32; 2],
    background: [u8; 4],
    sprites: Vec<Sprite>,
    opacity: f32,
}

pub fn install(state: &egui_wgpu::RenderState, ctx: &egui::Context) {
    let device = &state.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Design compositor"),
        source: wgpu::ShaderSource::Wgsl(include_str!("composite.wgsl").into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Design layers"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            texture_binding(1),
            texture_binding(2),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Design compositor"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = |entry, format, blend| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(entry),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        })
    };
    let layer = pipeline("fs_layer", wgpu::TextureFormat::Rgba8Unorm, None);
    let display = pipeline(
        "fs_display",
        state.target_format,
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
    );
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Design filtering"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    state.renderer.write().callback_resources.insert(Resources {
        layer,
        display,
        layout,
        sampler,
        images: HashMap::new(),
        instances: HashMap::new(),
    });
    ctx.data_mut(|data| data.insert_temp(egui::Id::NULL.with("design_gpu_ready"), true));
}
fn texture_binding(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}
pub(super) fn paint(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene: &DesignDocument,
    theme: Option<&crate::core::models::PresentationTheme>,
) -> bool {
    if !painter.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::NULL.with("design_gpu_ready"))
            .unwrap_or(false)
    }) {
        return false;
    }
    if !rect.is_positive() || !painter.clip_rect().intersects(rect) {
        return true;
    }
    let ctx = painter.ctx();
    let viewport = ctx.viewport_id();
    let pass = ctx.cumulative_pass_nr();
    let key = ctx.data_mut(|data| {
        let slots =
            data.get_temp_mut_or_default::<Slots>(egui::Id::NULL.with("design_slots").with(viewport));
        if slots.pass != pass {
            slots.pass = pass;
            slots.next = 0;
        }
        let key = Key {
            viewport,
            slot: slots.next,
        };
        slots.next += 1;
        key
    });
    let pixels = ctx.pixels_per_point();
    let scale = (4096.0 / (rect.width().max(rect.height()) * pixels)).min(1.0);
    let size = [
        (rect.width() * pixels * scale).round().max(1.0) as u32,
        (rect.height() * pixels * scale).round().max(1.0) as u32,
    ];
    let sprites = sprites::layers_at(scene, rect.aspect_ratio(), theme, Some(ctx), size[1]);
    painter.add(egui_wgpu::Callback::new_paint_callback(
        rect,
        Callback {
            key,
            frame: ctx.cumulative_frame_nr(),
            size,
            background: if scene.background_mode == "color" {
                scene.background
            } else {
                [0; 4]
            },
            sprites,
            opacity: painter.opacity(),
        },
    ));
    true
}
fn texture(device: &wgpu::Device, size: [u32; 2], usage: wgpu::TextureUsages) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Design layer texture"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    })
}
fn bind(
    device: &wgpu::Device,
    resources: &Resources,
    params: Params,
    previous: &wgpu::TextureView,
    source: &wgpu::TextureView,
) -> wgpu::BindGroup {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Design layer parameters"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Design layer"),
        layout: &resources.layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(previous),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(source),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&resources.sampler),
            },
        ],
    })
}
impl egui_wgpu::CallbackTrait for Callback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(resources) = resources.get_mut::<Resources>() else {
            return vec![];
        };
        if resources.images.len() > 128 {
            resources
                .images
                .retain(|_, (_, frame)| *frame + 2 >= self.frame);
        }
        for sprite in &self.sprites {
            let entry = resources.images.entry(sprite.key).or_insert_with(|| {
                let size = [sprite.image.width(), sprite.image.height()];
                let tex = texture(
                    device,
                    size,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                );
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    sprite.image.as_raw(),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(size[0] * 4),
                        rows_per_image: Some(size[1]),
                    },
                    wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                );
                (tex.create_view(&Default::default()), self.frame)
            });
            entry.1 = self.frame;
        }
        let views = if let Some(instance) = resources
            .instances
            .get(&self.key)
            .filter(|instance| instance.size == self.size)
        {
            instance.views.clone()
        } else {
            std::array::from_fn(|_| {
                texture(
                    device,
                    self.size,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .create_view(&Default::default())
            })
        };
        let alpha = self.background[3] as f64 / 255.0;
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Design clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &views[0],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.background[0] as f64 / 255.0 * alpha,
                            g: self.background[1] as f64 / 255.0 * alpha,
                            b: self.background[2] as f64 / 255.0 * alpha,
                            a: alpha,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        let mut groups = Vec::new();
        for (index, sprite) in self.sprites.iter().enumerate() {
            let params = Params {
                values: [
                    sprite.rect,
                    [
                        sprite.opacity,
                        match sprite.blend {
                            BlendMode::Normal => 0.0,
                            BlendMode::Multiply => 1.0,
                            BlendMode::Screen => 2.0,
                            BlendMode::Overlay => 3.0,
                            BlendMode::Darken => 4.0,
                            BlendMode::Lighten => 5.0,
                            BlendMode::ColorDodge => 6.0,
                            BlendMode::ColorBurn => 7.0,
                            BlendMode::HardLight => 8.0,
                            BlendMode::SoftLight => 9.0,
                            BlendMode::Difference => 10.0,
                            BlendMode::Exclusion => 11.0,
                        },
                        sprite.fit,
                        if sprite.shape { 1.0 } else { 0.0 },
                    ],
                    [
                        sprite.radius,
                        sprite.image.width() as f32 / sprite.image.height() as f32,
                        sprite.blur,
                        sprite.rotation,
                    ],
                    sprite.tint.map(|c| c as f32 / 255.0),
                    [self.size[0] as f32, self.size[1] as f32, 0.0, 0.0],
                    [sprite.content[0], sprite.content[1], sprite.shape_kind, sprite.line_width],
                    [sprite.stroke[0], sprite.stroke[1], 0.0, 0.0],
                    sprite.stroke_color.map(|c| c as f32 / 255.0),
                    sprite.shadow,
                    sprite.shadow_color.map(|c| c as f32 / 255.0),
                ],
            };
            let group = bind(
                device,
                resources,
                params,
                &views[index % 2],
                &resources.images[&sprite.key].0,
            );
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Design blend pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &views[(index + 1) % 2],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(&resources.layer);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, 0..1);
            }
            groups.push(group);
        }
        let output = &views[self.sprites.len() % 2];
        let display = bind(
            device,
            resources,
            Params {
                values: {
                    let mut values = [[0.0; 4]; 10];
                    values[1][0] = self.opacity;
                    values
                },
            },
            output,
            output,
        );
        resources.instances.insert(
            self.key,
            Instance {
                views,
                size: self.size,
                groups,
                display,
            },
        );
        vec![]
    }
    fn paint(
        &self,
        _: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        if let Some(resources) = resources.get::<Resources>()
            && let Some(instance) = resources.instances.get(&self.key)
        {
            let _ = &instance.groups;
            pass.set_pipeline(&resources.display);
            pass.set_bind_group(0, &instance.display, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn callback_creation_does_not_lock_the_context_recursively() {
        let ctx = eframe::egui::Context::default();
        ctx.data_mut(|data| data.insert_temp(eframe::egui::Id::NULL.with("design_gpu_ready"), true));
        let scene = crate::core::design::DesignDocument::template(2);
        let mut result = ctx.run_ui(eframe::egui::RawInput::default(), |ui| {
            super::paint(
                ui.painter(),
                eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(640.0, 360.0),
                ),
                &scene,
                None,
            );
        });
        assert!(
            result
                .shapes
                .iter()
                .any(|shape| matches!(shape.shape, eframe::egui::Shape::Callback(_)))
        );
        result.textures_delta.clear();
    }
    #[test]
    fn compositor_shader_validates() {
        let module = naga::front::wgsl::parse_str(include_str!("composite.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}
