use bevy::asset::HandleUntyped;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::ExtractedCamera,
        render_graph::{self, Node, NodeRunError, SlotInfo, SlotType},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor,
            PipelineCache, SamplerBindingType, ShaderStages, StorageTextureAccess, TextureFormat,
            TextureSampleType, TextureViewDimension,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewDepthTexture},
    },
};

use super::view::{GBuffer, GBuffers};

pub struct ZMipNode {
    query: QueryState<
        (
            &'static ExtractedCamera,
            &'static Camera3d,
            &'static GBuffer,
            &'static PrimitiveVisibilityBindGroup,
        ),
        With<ExtractedView>,
    >,
}
impl ZMipNode {
    pub const NAME: &'static str = "visibility_counter";
    pub const IN_VIEW: &'static str = "view";
    pub(crate) const Z_BUFFER_BLIT_HANDLE: HandleUntyped =
        HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9911374759819384610);
    pub(crate) const MIPMAP_GEN_HANDLE: HandleUntyped =
        HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9911374759819384611);

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for ZMipNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(ZMipNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (_camera, _view, _gbuffer, bind_group) = match self.query.get_manual(world, view_entity)
        {
            Ok(query) => query,
            Err(_) => {
                return Ok(());
            } // No window
        };

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ZMipPipeline>();

        if let (Some(init_pipeline), Some(mipmap_pipeline)) = (
            pipeline_cache.get_compute_pipeline(pipeline.pipeline),
            pipeline_cache.get_compute_pipeline(pipeline.mipmap_pipeline),
        ) {
            let mut pass = render_context
                .command_encoder
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_pipeline(init_pipeline);
            pass.set_bind_group(0, &bind_group.depth_bind_group, &[]);
            pass.dispatch_workgroups(1024 / 8, 1024 / 8, 1);

            pass.set_pipeline(mipmap_pipeline);
            for i in 0..6 {
                pass.set_bind_group(0, &bind_group.mipmap_bind_groups[i], &[]);
                let image_size = 1024 / (2 << i);
                pass.dispatch_workgroups(image_size / 8, image_size / 8, 1);
            }
        }
        Ok(())
    }
}

#[derive(Resource)]
pub struct ZMipPipeline {
    texture_bind_group_layout: BindGroupLayout,
    mipmap_bind_group_layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
    mipmap_pipeline: CachedComputePipelineId,
}

impl FromWorld for ZMipPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            // The input
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Depth,
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            // The output buffer
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: TextureFormat::R32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            // The sampler
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let mipmap_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            // The input
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            // The output buffer
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: TextureFormat::R32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            // The sampler
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![texture_bind_group_layout.clone()]),
            shader: ZMipNode::Z_BUFFER_BLIT_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "main".into(),
        });
        let mipmap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![mipmap_bind_group_layout.clone()]),
            shader: ZMipNode::MIPMAP_GEN_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "main".into(),
        });

        ZMipPipeline {
            texture_bind_group_layout,
            mipmap_bind_group_layout,
            pipeline,
            mipmap_pipeline,
        }
    }
}

#[derive(Component)]
struct PrimitiveVisibilityBindGroup {
    depth_bind_group: BindGroup,
    mipmap_bind_groups: Vec<BindGroup>,
}
pub(crate) fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ZMipPipeline>,
    render_device: Res<RenderDevice>,
    gbuffers: Res<GBuffers>,
    query: Query<(Entity, &GBuffer, &ViewDepthTexture)>,
) {
    for (entity, gbuffer, depth) in query.iter() {
        let depth_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&depth.view),
                }, // input
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&gbuffer.mipmap_views[0]),
                }, // output
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&gbuffers.sampler),
                }, // output
            ],
        });
        let mipmap_bind_groups = (0..6)
            .map(|i| {
                render_device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &pipeline.mipmap_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&gbuffer.mipmap_views[i]),
                        }, // input
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&gbuffer.mipmap_views[i + 1]),
                        }, // output
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::Sampler(&gbuffers.sampler),
                        }, // output
                    ],
                })
            })
            .collect();
        commands
            .entity(entity)
            .insert(PrimitiveVisibilityBindGroup {
                depth_bind_group,
                mipmap_bind_groups,
            });
    }
}
