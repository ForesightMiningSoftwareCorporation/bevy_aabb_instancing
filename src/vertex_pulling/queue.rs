use super::cuboid_cache::CuboidBufferCache;
use super::draw::DrawCuboids;
use super::pipeline::CuboidsPipeline;

use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::view::{ExtractedView, VisibleEntities};

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
