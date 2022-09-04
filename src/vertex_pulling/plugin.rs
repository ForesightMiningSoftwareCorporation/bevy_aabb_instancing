use super::buffer_cache::BufferCache;
use super::draw::{ClippingPlanesMeta, DrawCuboids, ViewMeta};
use super::extract::{extract_clipping_planes, extract_cuboids, CuboidsTransform};
use super::index_buffer::CuboidsIndexBuffer;
use super::pipeline::{CuboidsPipeline, CuboidsShaderDefs, VERTEX_PULLING_SHADER_HANDLE};
use super::prepare::{prepare_clipping_planes, prepare_cuboids, GpuClippingPlaneRanges};
use super::queue::{queue_cuboids, queue_cuboids_view_bind_group};

use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::{
    render_phase::AddRenderCommand,
    render_resource::{DynamicUniformBuffer, UniformBuffer},
    RenderApp, RenderStage,
};

/// Renders the [`Cuboids`](crate::Cuboids) component using the "vertex pulling" technique.
#[derive(Default)]
pub struct VertexPullingRenderPlugin {
    pub outlines: bool,
}

impl Plugin for VertexPullingRenderPlugin {
    fn build(&self, app: &mut App) {
        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            VERTEX_PULLING_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("vertex_pulling.wgsl")),
        );

        let maybe_msaa = app.world.get_resource::<Msaa>().cloned();
        let render_app = app.sub_app_mut(RenderApp);

        if let Some(msaa) = maybe_msaa {
            render_app.insert_resource(msaa.clone());
        }
        let mut shader_defs = CuboidsShaderDefs::default();
        if self.outlines {
            shader_defs.enable_outlines();
        }
        render_app.insert_resource(shader_defs);

        render_app
            .add_render_command::<Opaque3d, DrawCuboids>()
            .init_resource::<CuboidsPipeline>()
            .init_resource::<BufferCache>()
            .init_resource::<DynamicUniformBuffer<CuboidsTransform>>()
            .init_resource::<UniformBuffer<GpuClippingPlaneRanges>>()
            .init_resource::<ViewMeta>()
            .init_resource::<ClippingPlanesMeta>()
            .insert_resource(CuboidsIndexBuffer::new())
            .add_system_to_stage(RenderStage::Extract, extract_cuboids)
            .add_system_to_stage(RenderStage::Extract, extract_clipping_planes)
            .add_system_to_stage(RenderStage::Prepare, prepare_cuboids)
            .add_system_to_stage(RenderStage::Prepare, prepare_clipping_planes)
            .add_system_to_stage(RenderStage::Queue, queue_cuboids_view_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_cuboids);
    }
}
