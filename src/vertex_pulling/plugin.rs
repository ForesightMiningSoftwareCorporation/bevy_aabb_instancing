use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use super::draw::{AuxiliaryMeta, DrawCuboids, TransformsMeta, ViewMeta};
use super::extract::{extract_clipping_planes, extract_cuboids};
use super::pipeline::{CuboidsPipeline, CuboidsShaderDefs, VERTEX_PULLING_SHADER_HANDLE};
use super::prepare::{
    prepare_auxiliary_bind_group, prepare_clipping_planes, prepare_color_options,
    prepare_cuboid_transforms, prepare_cuboids, prepare_cuboids_view_bind_group,
};
use super::primitive_visibility;
use super::queue::queue_cuboids;
use super::view::GBuffers;

use crate::ColorOptionsMap;

use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::render_graph::RenderGraph;
use bevy::render::{render_phase::AddRenderCommand, RenderApp, RenderStage};

/// Renders the [`Cuboids`](crate::Cuboids) component using the "vertex pulling" technique.
#[derive(Default)]
pub struct VertexPullingRenderPlugin {
    pub outlines: bool,
    pub culling: bool,
}

impl Plugin for VertexPullingRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorOptionsMap>();

        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            VERTEX_PULLING_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("vertex_pulling.wgsl")),
        );
        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            super::primitive_visibility::ZMipNode::Z_BUFFER_BLIT_HANDLE,
            Shader::from_wgsl(include_str!("visibility_counting.wgsl")),
        );
        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            super::primitive_visibility::ZMipNode::MIPMAP_GEN_HANDLE,
            Shader::from_wgsl(include_str!("mipmap_gen.wgsl")),
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
        if self.culling {
            shader_defs.enable_culling();
        }
        render_app.insert_resource(shader_defs);

        render_app
            .add_render_command::<Opaque3d, DrawCuboids>()
            .init_resource::<GBuffers>()
            .init_resource::<AuxiliaryMeta>()
            .init_resource::<CuboidBufferCache>()
            .init_resource::<CuboidsPipeline>()
            .init_resource::<DynamicUniformBufferOfColorOptions>()
            .init_resource::<DynamicUniformBufferOfCuboidTransforms>()
            .init_resource::<TransformsMeta>()
            .init_resource::<UniformBufferOfGpuClippingPlaneRanges>()
            .init_resource::<ViewMeta>()
            .init_resource::<primitive_visibility::ZMipPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_cuboids)
            .add_system_to_stage(RenderStage::Extract, extract_clipping_planes)
            .add_system_to_stage(RenderStage::Prepare, prepare_color_options)
            .add_system_to_stage(RenderStage::Prepare, prepare_clipping_planes)
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_auxiliary_bind_group
                    .after(prepare_color_options)
                    .after(prepare_clipping_planes),
            )
            .add_system_to_stage(RenderStage::Prepare, super::view::prepare_view_targets)
            .add_system_to_stage(RenderStage::Prepare, prepare_cuboid_transforms)
            .add_system_to_stage(RenderStage::Prepare, prepare_cuboids)
            // HACK: prepare view bind group should happen in prepare phase, but
            // ViewUniforms resource is not ready until after prepare phase;
            // need system order/label exported from bevy
            .add_system_to_stage(RenderStage::Queue, prepare_cuboids_view_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_cuboids);

        if self.culling {
            render_app
                .add_system_to_stage(RenderStage::Queue, primitive_visibility::queue_bind_group);
            let visibility_node = primitive_visibility::ZMipNode::new(&mut render_app.world);

            let mut graph = render_app.world.resource_mut::<RenderGraph>();
            let draw_3d_graph = graph
                .get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME)
                .unwrap();
            let visibility_counter_node =
                draw_3d_graph.add_node(primitive_visibility::ZMipNode::NAME, visibility_node);
            draw_3d_graph
                .add_node_edge(
                    bevy::core_pipeline::core_3d::graph::node::MAIN_PASS,
                    visibility_counter_node,
                )
                .unwrap();

            draw_3d_graph
                .add_slot_edge(
                    draw_3d_graph.input_node().unwrap().id,
                    bevy::core_pipeline::core_3d::graph::input::VIEW_ENTITY,
                    primitive_visibility::ZMipNode::NAME,
                    primitive_visibility::ZMipNode::IN_VIEW,
                )
                .unwrap();
        }
    }
}
