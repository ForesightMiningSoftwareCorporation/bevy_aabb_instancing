use super::cuboid_cache::CuboidBufferCache;
use super::draw::{DrawCuboids, ViewMeta};
use super::pipeline::CuboidsPipeline;

use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::view::VisibleEntities;
use bevy::render::{
    render_resource::{BindGroupDescriptor, BindGroupEntry},
    renderer::RenderDevice,
    view::{ExtractedView, ViewUniforms},
};

pub(crate) fn queue_cuboids_view_bind_group(
    render_device: Res<RenderDevice>,
    cuboids_pipeline: Res<CuboidsPipeline>,
    mut view_meta: ResMut<ViewMeta>,
    view_uniforms: Res<ViewUniforms>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        view_meta.cuboids_view_bind_group =
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding,
                }],
                label: Some("cuboids_view_bind_group"),
                layout: &cuboids_pipeline.view_layout,
            }));
    }
}

pub(crate) fn queue_cuboids(
    cuboids_pipeline: Res<CuboidsPipeline>,
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    buffer_cache: Res<CuboidBufferCache>,
    mut views: Query<(&ExtractedView, &VisibleEntities, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_cuboids = opaque_3d_draw_functions
        .read()
        .get_id::<DrawCuboids>()
        .unwrap();

    for (view, visible_entities, mut opaque_phase) in views.iter_mut() {
        let inverse_view_matrix = view.transform.compute_matrix().inverse();
        let inverse_view_row_2 = inverse_view_matrix.row(2);
        for &entity in &visible_entities.entities {
            if let Some(entry) = buffer_cache.get(entity) {
                if entry.is_enabled() {
                    let entity_center = entry.aabb().center;
                    opaque_phase.add(Opaque3d {
                        pipeline: cuboids_pipeline.pipeline_id,
                        entity,
                        distance: inverse_view_row_2.dot(entity_center.extend(1.0)),
                        draw_function: draw_cuboids,
                    });
                }
            }
        }
    }
}
