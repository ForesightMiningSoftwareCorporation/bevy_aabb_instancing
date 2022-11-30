use crate::clipping_planes::GpuClippingPlaneRanges;
use crate::{cuboids::CuboidsTransform, ColorOptions};

use bevy::render::render_resource::{SamplerBindingType, TextureSampleType, TextureViewDimension};
use bevy::{
    prelude::*,
    reflect::TypeUuid,
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
        texture::BevyDefault,
        view::ViewUniform,
    },
};

#[derive(Resource)]
pub(crate) struct CuboidsPipeline {
    pub pipeline_id: CachedRenderPipelineId,
    pub aux_layout: BindGroupLayout,
    pub cuboids_layout: BindGroupLayout,
    pub transforms_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
}

pub(crate) const VERTEX_PULLING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17343092250772987267);

impl FromWorld for CuboidsPipeline {
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
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
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
                        min_binding_size: Some(ColorOptions::min_size()),
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

        let sample_count = world.get_resource::<Msaa>().map(|m| m.samples).unwrap_or(1);
        let shader_defs = world.resource::<CuboidsShaderDefs>();
        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("cuboids_pipeline".into()),
            layout: Some(vec![
                view_layout.clone(),
                aux_layout.clone(),
                transforms_layout.clone(),
                cuboids_layout.clone(),
            ]),
            vertex: VertexState {
                shader: VERTEX_PULLING_SHADER_HANDLE.typed(),
                shader_defs: shader_defs.vertex.clone(),
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: VERTEX_PULLING_SHADER_HANDLE.typed(),
                shader_defs: shader_defs.fragment.clone(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
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
            }),
            multisample: MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(pipeline_descriptor);

        Self {
            pipeline_id,
            view_layout,
            aux_layout,
            cuboids_layout,
            transforms_layout,
        }
    }
}

#[derive(Clone, Default, Resource)]
pub(crate) struct CuboidsShaderDefs {
    pub vertex: Vec<String>,
    pub fragment: Vec<String>,
}

impl CuboidsShaderDefs {
    pub fn enable_outlines(&mut self) {
        self.vertex.push("OUTLINES".into());
        self.fragment.push("OUTLINES".into());
    }
    pub fn enable_culling(&mut self) {
        self.vertex.push("CULLING".into());
    }
}
