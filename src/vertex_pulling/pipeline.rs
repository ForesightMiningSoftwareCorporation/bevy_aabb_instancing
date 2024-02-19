use crate::clipping_planes::GpuClippingPlaneRanges;
use crate::{cuboids::CuboidsTransform, CuboidMaterial};

use bevy::render::render_resource::ShaderDefVal;
use bevy::render::texture::BevyDefault;
use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, BufferSize, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, FragmentState,
            FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            RenderPipelineDescriptor, ShaderStages, ShaderType, StencilFaceState, StencilState,
            TextureFormat, VertexState,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

#[derive(Resource)]
pub(crate) struct CuboidsPipelines {
    pub pipeline_id: CachedRenderPipelineId,
    pub hdr_pipeline_id: CachedRenderPipelineId,

    pub aux_layout: BindGroupLayout,
    pub cuboids_layout: BindGroupLayout,
    pub transforms_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
}

pub(crate) const VERTEX_PULLING_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(17343092250772987267);

impl FromWorld for CuboidsPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("cuboids_view_layout"),
            entries: &[
                // View
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(ViewUniform::min_size().get()),
                    },
                    count: None,
                },
            ],
        });

        let aux_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("aux_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(CuboidMaterial::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuClippingPlaneRanges::min_size()),
                    },
                    count: None,
                },
            ],
        });

        let transforms_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("transforms_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        // We always have a single transform for each instance buffer.
                        min_binding_size: Some(CuboidsTransform::min_size()),
                    },
                    count: None,
                }],
            });

        let cuboids_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("cuboid_instances_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(0),
                },
                count: None,
            }],
        });

        let sample_count = world.resource::<Msaa>().samples();
        let shader_defs = world.resource::<CuboidsShaderDefs>();

        let layout = vec![
            view_layout.clone(),
            aux_layout.clone(),
            transforms_layout.clone(),
            cuboids_layout.clone(),
        ];
        let vertex = VertexState {
            shader: VERTEX_PULLING_SHADER_HANDLE,
            shader_defs: shader_defs.vertex.clone(),
            entry_point: "vertex".into(),
            buffers: vec![],
        };
        let fragment_target = |texture_format| FragmentState {
            shader: VERTEX_PULLING_SHADER_HANDLE,
            shader_defs: shader_defs.fragment.clone(),
            entry_point: "fragment".into(),
            targets: vec![Some(ColorTargetState {
                format: texture_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        };
        let primitive = PrimitiveState {
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
        };
        let depth_stencil = Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Greater,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
        });
        let multisample = MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("cuboids_pipeline".into()),
            layout: layout.clone(),
            vertex: vertex.clone(),
            fragment: Some(fragment_target(TextureFormat::bevy_default())),
            primitive,
            depth_stencil: depth_stencil.clone(),
            multisample,
            push_constant_ranges: Vec::new(),
        };

        let hdr_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("cuboids_hdr_pipeline".into()),
            layout,
            vertex,
            fragment: Some(fragment_target(TextureFormat::Rgba16Float)),
            primitive,
            depth_stencil,
            multisample,
            push_constant_ranges: Vec::new(),
        };

        let pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(pipeline_descriptor);
        let hdr_pipeline_id = pipeline_cache.queue_render_pipeline(hdr_pipeline_descriptor);

        Self {
            pipeline_id,
            hdr_pipeline_id,
            view_layout,
            aux_layout,
            cuboids_layout,
            transforms_layout,
        }
    }
}

#[derive(Clone, Default, Resource)]
pub(crate) struct CuboidsShaderDefs {
    pub vertex: Vec<ShaderDefVal>,
    pub fragment: Vec<ShaderDefVal>,
}

impl CuboidsShaderDefs {
    pub fn enable_outlines(&mut self) {
        self.vertex.push("OUTLINES".into());
        self.fragment.push("OUTLINES".into());
    }
}
