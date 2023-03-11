use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use super::draw::{AuxiliaryMeta, DrawCuboids, TransformsMeta, ViewMeta};
use super::extract::{extract_clipping_planes, extract_cuboids};
use super::pipeline::{CuboidsPipeline, CuboidsShaderDefs, VERTEX_PULLING_SHADER_HANDLE};
use super::prepare::{
    prepare_auxiliary_bind_group, prepare_clipping_planes, prepare_color_options,
    prepare_cuboid_transforms, prepare_cuboids, prepare_cuboids_view_bind_group,
};
use super::queue::queue_cuboids;
use crate::ColorOptionsMap;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::RenderSet;
use bevy::render::{render_phase::AddRenderCommand, RenderApp};

/// Renders the [`Cuboids`](crate::Cuboids) component using the "vertex pulling" technique.
#[derive(Default)]
pub struct VertexPullingRenderPlugin {
    pub outlines: bool,
}

impl Plugin for VertexPullingRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorOptionsMap>();

        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            VERTEX_PULLING_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("vertex_pulling.wgsl")),
        );
        {
            use super::index_buffer::{CuboidsIndexBuffer, CUBE_INDICES_HANDLE};
            use bevy::render::render_asset::RenderAssetPlugin;
            app.add_asset::<CuboidsIndexBuffer>()
                .add_plugin(RenderAssetPlugin::<CuboidsIndexBuffer>::default());
            app.world
                .resource_mut::<Assets<CuboidsIndexBuffer>>()
                .set_untracked(CUBE_INDICES_HANDLE, CuboidsIndexBuffer);
        }

        let maybe_msaa = app.world.get_resource::<Msaa>().cloned();
        let render_app = app.sub_app_mut(RenderApp);

        if let Some(msaa) = maybe_msaa {
            render_app.insert_resource(msaa);
        }
        let mut shader_defs = CuboidsShaderDefs::default();
        if self.outlines {
            shader_defs.enable_outlines();
        }
        render_app.insert_resource(shader_defs);

        render_app
            .add_render_command::<Opaque3d, DrawCuboids>()
            .init_resource::<AuxiliaryMeta>()
            .init_resource::<CuboidBufferCache>()
            .init_resource::<CuboidsPipeline>()
            .init_resource::<DynamicUniformBufferOfColorOptions>()
            .init_resource::<DynamicUniformBufferOfCuboidTransforms>()
            .init_resource::<TransformsMeta>()
            .init_resource::<UniformBufferOfGpuClippingPlaneRanges>()
            .init_resource::<ViewMeta>()
            .add_systems((extract_cuboids, extract_clipping_planes).in_schedule(ExtractSchedule))
            .add_systems(
                (
                    prepare_color_options,
                    prepare_clipping_planes,
                    prepare_auxiliary_bind_group
                        .after(prepare_color_options)
                        .after(prepare_clipping_planes),
                    prepare_cuboid_transforms,
                    prepare_cuboids,
                )
                    .in_set(RenderSet::Prepare),
            )
            // HACK: prepare view bind group should happen in prepare phase, but
            // ViewUniforms resource is not ready until after prepare phase;
            // need system order/label exported from bevy
            .add_systems((prepare_cuboids_view_bind_group, queue_cuboids).in_set(RenderSet::Queue));
    }
}
