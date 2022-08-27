use super::{extract::CuboidsTransform, prepare::GpuClippingPlaneRanges};

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

pub(crate) struct CuboidsPipeline {
    pub pipeline_id: CachedRenderPipelineId,
    pub view_layout: BindGroupLayout,
    pub clipping_planes_layout: BindGroupLayout,
    pub cuboids_layout: BindGroupLayout,
    pub transforms_layout: BindGroupLayout,
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
            ],
        });

        let clipping_planes_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("clipping_planes_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuClippingPlaneRanges::min_size()),
                    },
                    count: None,
                }],
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
                        min_binding_size: BufferSize::new(CuboidsTransform::min_size().get()),
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
        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("cuboids_pipeline".into()),
            layout: Some(vec![
                view_layout.clone(),
                clipping_planes_layout.clone(),
                transforms_layout.clone(),
                cuboids_layout.clone(),
            ]),
            vertex: VertexState {
                shader: VERTEX_PULLING_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: VERTEX_PULLING_SHADER_HANDLE.typed(),
                shader_defs: vec![],
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
        });

        Self {
            pipeline_id,
            view_layout,
            clipping_planes_layout,
            cuboids_layout,
            transforms_layout,
        }
    }
}
