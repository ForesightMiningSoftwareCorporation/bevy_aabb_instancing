use std::borrow::Cow;

use bevy::{render::{render_resource::{BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureFormat, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BufferBindingType, ComputePassDescriptor, BindGroupDescriptor, BindingResource, BindGroupEntry, BindGroup, BufferBinding, TextureSampleType, SamplerBindingType}, renderer::{RenderDevice, RenderContext}, render_graph::{Node, self, SlotInfo, SlotType, NodeRunError}, view::{ExtractedView, ViewDepthTexture}, camera::ExtractedCamera, render_asset::RenderAssets}, prelude::*, reflect::TypeUuid, ecs::query};
use bevy::asset::HandleUntyped;

use super::{view::{GBuffer, GBuffers}, draw::ViewMeta, cuboid_cache::{CachedCuboidBuffers, CuboidBufferCache}};

pub struct VisibilityCounterNode {
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
impl VisibilityCounterNode {
    pub const NAME: &'static str = "visibility_counter";
    pub const IN_VIEW: &'static str = "view";
    pub(crate) const VISIBILITY_COUNTING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9911374759819384610);
    
    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for VisibilityCounterNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(VisibilityCounterNode::IN_VIEW, SlotType::Entity)]
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
        let (camera, view, gbuffer, bind_group) =
            match self.query.get_manual(world, view_entity) {
                Ok(query) => query,
                Err(_) => {
                    return Ok(());
                } // No window
            };


        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<VisibilityCounterPipeline>();

        if let Some(init_pipeline) = pipeline_cache
            .get_compute_pipeline(pipeline.pipeline) {
                let mut pass = render_context
                .command_encoder
                .begin_compute_pass(&ComputePassDescriptor::default());
    
                pass.set_bind_group(0, &bind_group.0, &[]);

                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(1024 / 8, 1024 / 8, 1);
            }
        Ok(())
    }
}

pub struct VisibilityCounterPipeline {
    texture_bind_group_layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for VisibilityCounterPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry { // The input
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Depth,
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry { // The output buffer
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture  {
                                access: StorageTextureAccess::WriteOnly,
                                format: TextureFormat::R32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry { // The output buffer
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        }
                    ],
                });
        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![texture_bind_group_layout.clone()]),
            shader: VisibilityCounterNode::VISIBILITY_COUNTING_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: "main".into(),
        });

        VisibilityCounterPipeline {
            texture_bind_group_layout,
            pipeline
        }
    }
}


#[derive(Component)]
struct PrimitiveVisibilityBindGroup(BindGroup);
pub(crate) fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<VisibilityCounterPipeline>,
    render_device: Res<RenderDevice>,
    cached_cuboid_buffers: Res<CuboidBufferCache>,
    gbuffers: Res<GBuffers>,
    query: Query<(Entity, &GBuffer, &ViewDepthTexture)>,
) {
    for (entity, gbuffer, depth) in query.iter() {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
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
        commands.entity(entity).insert(PrimitiveVisibilityBindGroup(bind_group));
    }
}
