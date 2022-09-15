use super::buffer_cache::{BufferCache, GpuCuboidBuffers};
use super::draw::{ClippingPlanesMeta, TransformsMeta};
use super::extract::RenderCuboids;
use super::index_buffer::CuboidsIndexBuffer;
use super::pipeline::CuboidsPipeline;
use crate::clipping_planes::GpuClippingPlaneRange;

use crate::cuboids::{Cuboid, CuboidsTransform};
use bevy::render::render_resource::{ShaderType, UniformBuffer};
use bevy::{
    prelude::*,
    render::{
        primitives::Aabb,
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, DynamicUniformBuffer, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

#[derive(Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRanges {
    ranges: [GpuClippingPlaneRange; 3],
    num_ranges: u32,
}

pub(crate) fn prepare_clipping_planes(
    pipeline: Res<CuboidsPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut clipping_meta: ResMut<ClippingPlanesMeta>,
    mut clipping_plane_uniform: ResMut<UniformBuffer<GpuClippingPlaneRanges>>,
    extracted_clipping_planes: Query<&GpuClippingPlaneRange>,
) {
    let mut iter = extracted_clipping_planes.iter();
    let mut gpu_planes = GpuClippingPlaneRanges::default();
    for plane in iter.by_ref() {
        gpu_planes.ranges[gpu_planes.num_ranges as usize] = plane.clone();
        gpu_planes.num_ranges += 1;
        if gpu_planes.num_ranges == 3 {
            break;
        }
    }
    if iter.next().is_some() {
        warn!("Too many GpuClippingPlaneRanges entities, at most 3 are supported");
    }
    clipping_plane_uniform.set(gpu_planes);
    clipping_plane_uniform.write_buffer(&render_device, &render_queue);
    let planes_binding = clipping_plane_uniform.binding().unwrap();
    clipping_meta.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("clipping_planes_bind_group"),
        layout: &pipeline.clipping_planes_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: planes_binding,
        }],
    }));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_cuboids(
    mut transform_indices_scratch: Local<Vec<u32>>,
    pipeline: Res<CuboidsPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut buffer_cache: ResMut<BufferCache>,
    mut transforms_meta: ResMut<TransformsMeta>,
    mut transform_uniforms: ResMut<DynamicUniformBuffer<CuboidsTransform>>,
    mut index_buffer: ResMut<CuboidsIndexBuffer>,
    mut render_cuboids: Query<(Entity, &mut RenderCuboids, &CuboidsTransform, &Aabb)>,
) {
    let create_instance_buffer_span =
        bevy::log::info_span!("prepare_cuboids::create_instance_buffer");
    let create_bind_group_span = bevy::log::info_span!("prepare_cuboids::create_bind_group");
    let grow_index_buffer_span = bevy::log::info_span!("prepare_cuboids::grow_index_buffer");
    let write_transform_buffer_span =
        bevy::log::info_span!("prepare_cuboids::write_transform_buffer");

    // This seems a little hacky. Need to write the buffer early so we have a binding to use in the loop below.
    transform_uniforms.clear();
    transform_indices_scratch.clear();
    write_transform_buffer_span.in_scope(|| {
        for (_, _, transform, _) in render_cuboids.iter() {
            transform_indices_scratch.push(transform_uniforms.push(transform.clone()));
        }
        transform_uniforms.write_buffer(&render_device, &render_queue);
    });
    let transforms_binding = if let Some(b) = transform_uniforms.binding() {
        b
    } else {
        assert!(transform_uniforms.is_empty());
        return;
    };
    transforms_meta.transform_buffer_bind_group = create_bind_group_span.in_scope(|| {
        Some(render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("gpu_cuboids_transforms_bind_group"),
            layout: &pipeline.transforms_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: transforms_binding,
            }],
        }))
    });

    for ((entity, mut cuboids, _, aabb), &transform_index) in render_cuboids
        .iter_mut()
        .zip(transform_indices_scratch.iter())
    {
        match &mut *cuboids {
            RenderCuboids::UpdateCuboids {
                cuboids: new_cuboids,
                is_visible,
            } => {
                assert!(!new_cuboids.instances.is_empty());
                let num_cuboids = new_cuboids.instances.len().try_into().unwrap();

                grow_index_buffer_span.in_scope(|| {
                    index_buffer.grow_to_fit_num_cuboids(
                        num_cuboids,
                        &render_device,
                        &render_queue,
                    );
                });

                let mut instance_buffer = StorageBuffer::<Vec<Cuboid>>::default();
                create_instance_buffer_span.in_scope(|| {
                    let _ = std::mem::replace(
                        instance_buffer.get_mut(),
                        std::mem::take(&mut new_cuboids.instances),
                    );
                    instance_buffer.write_buffer(&render_device, &render_queue);
                });
                let instance_buffer_bind_group = create_bind_group_span.in_scope(|| {
                    render_device.create_bind_group(&BindGroupDescriptor {
                        label: Some("gpu_cuboids_instance_buffer_bind_group"),
                        layout: &pipeline.cuboids_layout,
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: instance_buffer.binding().unwrap(),
                        }],
                    })
                });

                buffer_cache.insert(
                    entity,
                    aabb.clone(),
                    *is_visible,
                    GpuCuboidBuffers {
                        _instance_buffer: instance_buffer,
                        instance_buffer_bind_group,
                        transform_index,
                        num_cuboids,
                    },
                );
            }
            RenderCuboids::UseCachedCuboids => {
                let entry = buffer_cache.get_mut(entity).unwrap();
                entry.buffers_mut().transform_index = transform_index;
            }
        }
    }
}
