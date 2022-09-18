use super::cuboid_cache::{CuboidBufferCache, GpuCuboidBuffers};
use super::draw::{AuxiliaryMeta, TransformsMeta, ViewMeta};
use super::extract::RenderCuboids;
use super::index_buffer::CuboidsIndexBuffer;
use super::pipeline::CuboidsPipeline;
use crate::clipping_planes::GpuClippingPlaneRange;
use crate::cuboids::{Cuboid, CuboidsTransform};
use crate::{ColorOptions, ColorOptionsUniformIndex};

use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroupDescriptor, BindGroupEntry},
        render_resource::{DynamicUniformBuffer, StorageBuffer},
        render_resource::{ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
    },
};

#[derive(Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRanges {
    ranges: [GpuClippingPlaneRange; 3],
    num_ranges: u32,
}

pub(crate) fn prepare_clipping_planes(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
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
}

pub(crate) fn prepare_color_options(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut color_options_uniforms: ResMut<DynamicUniformBuffer<ColorOptions>>,
) {
    // Values already pushed in extract stage.
    color_options_uniforms.write_buffer(&render_device, &render_queue);
}

pub(crate) fn prepare_auxiliary_bind_group(
    pipeline: Res<CuboidsPipeline>,
    render_device: Res<RenderDevice>,
    mut aux_meta: ResMut<AuxiliaryMeta>,
    clipping_plane_uniform: Res<UniformBuffer<GpuClippingPlaneRanges>>,
    color_options_uniform: Res<DynamicUniformBuffer<ColorOptions>>,
) {
    if let (Some(color_binding), Some(planes_binding)) = (
        color_options_uniform.binding(),
        clipping_plane_uniform.binding(),
    ) {
        aux_meta.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("auxiliary_bind_group"),
            layout: &pipeline.aux_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: color_binding,
                },
                BindGroupEntry {
                    binding: 1,
                    resource: planes_binding,
                },
            ],
        }));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_cuboids(
    pipeline: Res<CuboidsPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut cuboid_buffers: ResMut<CuboidBufferCache>,
    mut transforms_meta: ResMut<TransformsMeta>,
    mut transform_uniforms: ResMut<DynamicUniformBuffer<CuboidsTransform>>,
    mut index_buffer: ResMut<CuboidsIndexBuffer>,
    mut render_cuboids: Query<(
        Entity,
        &mut RenderCuboids,
        &CuboidsTransform,
        &ColorOptionsUniformIndex,
    )>,
) {
    let create_instance_buffer_span =
        bevy::log::info_span!("prepare_cuboids::create_instance_buffer");
    let create_bind_group_span = bevy::log::info_span!("prepare_cuboids::create_bind_group");
    let grow_index_buffer_span = bevy::log::info_span!("prepare_cuboids::grow_index_buffer");
    let write_transform_buffer_span =
        bevy::log::info_span!("prepare_cuboids::write_transform_buffer");

    transform_uniforms.clear();

    for (entity, mut cuboids, transform, color_options_index) in render_cuboids.iter_mut() {
        let transform_index = transform_uniforms.push(transform.clone());

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

                cuboid_buffers.insert(
                    entity,
                    transform.position(),
                    *is_visible,
                    GpuCuboidBuffers {
                        _instance_buffer: instance_buffer,
                        instance_buffer_bind_group,
                        transform_index,
                        color_options_index: color_options_index.0,
                        num_cuboids,
                    },
                );
            }
            RenderCuboids::UseCachedCuboids => {
                let entry = cuboid_buffers.get_mut(entity).unwrap();
                entry.buffers_mut().transform_index = transform_index;
            }
        }
    }

    write_transform_buffer_span.in_scope(|| {
        transform_uniforms.write_buffer(&render_device, &render_queue);
    });
    if let Some(transforms_binding) = transform_uniforms.binding() {
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
    } else {
        assert!(transform_uniforms.is_empty());
    }
}

pub(crate) fn prepare_cuboids_view_bind_group(
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
