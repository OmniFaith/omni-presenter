use crate::core::plugins::{PluginShader, shader};
use eframe::egui_wgpu::wgpu;

pub(super) fn pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    layout: &wgpu::BindGroupLayout,
    definition: &PluginShader,
) -> Option<wgpu::RenderPipeline> {
    shader::validate(definition).ok()?;
    let source = shader::wgsl(definition);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("plugin background"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("plugin background layout"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    Some(
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("plugin background"),
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
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        }),
    )
}
