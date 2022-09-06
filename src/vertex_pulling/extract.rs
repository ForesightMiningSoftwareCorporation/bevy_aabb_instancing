use super::buffer_cache::BufferCache;
use crate::component::*;

use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType, Extract},
};

#[derive(Clone, Component, ShaderType)]
pub(crate) struct CuboidsTransform {
    pub matrix: Mat4,
    pub inv_matrix: Mat4,
}

impl CuboidsTransform {
    pub fn new(matrix: Mat4, inv_matrix: Mat4) -> Self {
        Self { matrix, inv_matrix }
    }

    pub fn from_matrix(m: Mat4) -> Self {
        Self::new(m, m.inverse())
    }
}

#[derive(Clone, Component)]
pub(crate) enum RenderCuboids {
    UpdateCuboids { cuboids: Cuboids, is_visible: bool },
    UseCachedCuboids,
}

#[allow(clippy::type_complexity)]
pub(crate) fn extract_cuboids(
    mut commands: Commands,
    mut render_cuboids_scratch: Local<Vec<(Entity, (RenderCuboids, CuboidsTransform, Aabb))>>,
    cuboids: Extract<
        Query<(
            Entity,
            &Cuboids,
            &GlobalTransform,
            &Aabb,
            Option<&ComputedVisibility>,
            Or<(Added<Cuboids>, Changed<Cuboids>)>,
        )>,
    >,
    mut buffer_cache: ResMut<BufferCache>,
) {
    render_cuboids_scratch.clear();

    for (entity, cuboids, transform, aabb, maybe_visibility, instance_buffer_needs_update) in
        cuboids.iter()
    {
        // Filter all entities that don't have any enabled instances.
        // If an entity went from some to none cuboids, then it will get
        // culled from the buffer cache.
        if cuboids.instances.is_empty() {
            continue;
        }

        let is_visible = maybe_visibility
            .map(ComputedVisibility::is_visible)
            .unwrap_or(true);

        let render_cuboids = if instance_buffer_needs_update {
            RenderCuboids::UpdateCuboids {
                cuboids: cuboids.clone(),
                is_visible,
            }
            // Buffer cache will get filled in RenderState::Prepare.
        } else {
            let entry = buffer_cache.get_mut(entity).unwrap();
            entry.keep_alive();
            if is_visible {
                entry.enable();
            } else {
                entry.disable();
            }
            RenderCuboids::UseCachedCuboids
        };

        // Need to spawn even if we're reusing cached cuboids, since transforms are overwritten every frame (in prepare phase).
        render_cuboids_scratch.push((
            entity,
            (
                render_cuboids,
                CuboidsTransform::from_matrix(transform.compute_matrix()),
                aabb.clone(),
            ),
        ));
    }
    buffer_cache.cull_entities();

    commands.insert_or_spawn_batch(render_cuboids_scratch.clone());
}

pub(crate) fn extract_clipping_planes(
    mut commands: Commands,
    clipping_planes: Extract<Query<(&ClippingPlaneRange, &GlobalTransform)>>,
) {
    commands.spawn_batch(
        clipping_planes
            .iter()
            .map(|(range, plane_tfm)| {
                let (_, rotation, translation) = plane_tfm.to_scale_rotation_translation();
                (GpuClippingPlaneRange {
                    origin: translation,
                    unit_normal: rotation * Vec3::Y,
                    min_sdist: range.min_sdist,
                    max_sdist: range.max_sdist,
                },)
            })
            .collect::<Vec<_>>(),
    );
}

#[derive(Clone, Component, Default, ShaderType)]
pub struct GpuClippingPlaneRange {
    pub origin: Vec3,
    pub unit_normal: Vec3,
    pub min_sdist: f32,
    pub max_sdist: f32,
}
