use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use crate::clipping_planes::*;
use crate::cuboids::*;
use crate::ColorOptionsId;
use crate::ColorOptionsMap;

use bevy::{prelude::*, render::Extract};

#[allow(clippy::type_complexity)]
pub(crate) fn extract_cuboids(
    cuboids: Extract<
        Query<(
            Entity,
            &Cuboids,
            &GlobalTransform,
            &ColorOptionsId,
            Option<&ComputedVisibility>,
            Or<(Added<Cuboids>, Changed<Cuboids>)>,
        )>,
    >,
    color_options: Extract<Res<ColorOptionsMap>>,
    mut color_options_uniforms: ResMut<DynamicUniformBufferOfColorOptions>,
    mut cuboid_buffers: ResMut<CuboidBufferCache>,
    mut transform_uniforms: ResMut<DynamicUniformBufferOfCuboidTransforms>,
) {
    transform_uniforms.clear();

    if color_options.is_empty() {
        warn!("Cannot draw Cuboids with empty ColorOptionsMap");
        return;
    }

    // First extract color options so we can assign dynamic uniform indices to cuboids.
    let color_options_indices = color_options.write_uniforms(&mut color_options_uniforms);

    for (
        entity,
        cuboids,
        transform,
        color_options_id,
        maybe_visibility,
        instance_buffer_needs_update,
    ) in cuboids.iter()
    {
        // Filter all entities that don't have any instances. If an entity went
        // from non-empty to empty, then it will get culled from the buffer
        // cache.
        if cuboids.instances.is_empty() {
            continue;
        }

        let transform = CuboidsTransform::from_matrix(transform.compute_matrix());

        let is_visible = maybe_visibility
            .map(ComputedVisibility::is_visible)
            .unwrap_or(true);

        let entry = cuboid_buffers.entries.entry(entity).or_default();
        if instance_buffer_needs_update {
            entry.instance_buffer.set(cuboids.instances.clone());
        }
        entry.color_options_index = color_options_indices[color_options_id.0].0;
        entry.dirty = instance_buffer_needs_update;
        entry.enabled = is_visible;
        entry.keep_alive = true;
        entry.position = transform.position();
        entry.transform_index = transform_uniforms.push(transform);
    }

    cuboid_buffers.cull_entities();
}

pub(crate) fn extract_clipping_planes(
    clipping_planes: Extract<Query<(&ClippingPlaneRange, &GlobalTransform)>>,
    mut clipping_plane_uniform: ResMut<UniformBufferOfGpuClippingPlaneRanges>,
) {
    let mut iter = clipping_planes.iter();
    let mut gpu_planes = GpuClippingPlaneRanges::default();
    for (range, transform) in iter.by_ref() {
        let (_, rotation, translation) = transform.to_scale_rotation_translation();
        gpu_planes.ranges[gpu_planes.num_ranges as usize] = GpuClippingPlaneRange {
            origin: translation,
            unit_normal: rotation * Vec3::X,
            min_sdist: range.min_sdist,
            max_sdist: range.max_sdist,
        };
        gpu_planes.num_ranges += 1;
        if gpu_planes.num_ranges as usize == MAX_CLIPPING_PLANES {
            break;
        }
    }
    if iter.next().is_some() {
        warn!(
            "Too many GpuClippingPlaneRanges entities, at most {MAX_CLIPPING_PLANES} are supported"
        );
    }
    clipping_plane_uniform.set(gpu_planes);
}
