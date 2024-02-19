use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use super::draw::{AuxiliaryMeta, TransformsMeta, ViewMeta};
use super::pipeline::CuboidsPipelines;

use bevy::render::render_resource::BindGroupEntries;
use bevy::{
    prelude::*,
    render::{
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
    },
};

pub(crate) fn prepare_clipping_planes(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut clipping_plane_uniform: ResMut<UniformBufferOfGpuClippingPlaneRanges>,
) {
    // Values already pushed in extract stage.
    clipping_plane_uniform.write_buffer(&render_device, &render_queue);
}

pub(crate) fn prepare_materials(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut material_uniforms: ResMut<DynamicUniformBufferOfCuboidMaterial>,
) {
    // Values already pushed in extract stage.
    material_uniforms.write_buffer(&render_device, &render_queue);
}

pub(crate) fn prepare_auxiliary_bind_group(
    pipeline: Res<CuboidsPipelines>,
    render_device: Res<RenderDevice>,
    mut aux_meta: ResMut<AuxiliaryMeta>,
    clipping_plane_uniform: Res<UniformBufferOfGpuClippingPlaneRanges>,
    material_uniform: Res<DynamicUniformBufferOfCuboidMaterial>,
) {
    if let (Some(color_binding), Some(planes_binding)) =
        (material_uniform.binding(), clipping_plane_uniform.binding())
    {
        aux_meta.bind_group = Some(render_device.create_bind_group(
            "auxiliary_bind_group",
            &pipeline.aux_layout,
            &BindGroupEntries::sequential((color_binding, planes_binding)),
        ));
    }
}

pub(crate) fn prepare_cuboid_transforms(
    pipeline: Res<CuboidsPipelines>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut transforms_meta: ResMut<TransformsMeta>,
    mut transform_uniforms: ResMut<DynamicUniformBufferOfCuboidTransforms>,
) {
    let write_transform_buffer_span =
        bevy::log::info_span!("prepare_cuboids::write_transform_buffer");
    write_transform_buffer_span.in_scope(|| {
        transform_uniforms.write_buffer(&render_device, &render_queue);
    });
    if let Some(transforms_binding) = transform_uniforms.binding() {
        let create_bind_group_span = bevy::log::info_span!("prepare_cuboids::create_bind_group");
        transforms_meta.transform_buffer_bind_group = create_bind_group_span.in_scope(|| {
            Some(render_device.create_bind_group(
                "gpu_cuboids_transforms_bind_group",
                &pipeline.transforms_layout,
                &BindGroupEntries::single(transforms_binding),
            ))
        });
    } else {
        assert!(transform_uniforms.is_empty());
    }
}

pub(crate) fn prepare_cuboids(
    pipeline: Res<CuboidsPipelines>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut cuboid_buffers: ResMut<CuboidBufferCache>,
) {
    let write_instance_buffer_span =
        bevy::log::info_span!("prepare_cuboids::write_instance_buffer");
    let create_bind_group_span = bevy::log::info_span!("prepare_cuboids::create_bind_group");

    // Write all dirty buffers from the cuboids cache.
    for entry in cuboid_buffers.entries.values_mut() {
        if !entry.dirty {
            assert!(entry.instance_buffer_bind_group.is_some());
            continue;
        }

        write_instance_buffer_span.in_scope(|| {
            entry
                .instance_buffer
                .write_buffer(&render_device, &render_queue);
        });

        entry.instance_buffer_bind_group = create_bind_group_span.in_scope(|| {
            Some(render_device.create_bind_group(
                "cuboids_instance_buffer_bind_group",
                &pipeline.cuboids_layout,
                &BindGroupEntries::single(entry.instance_buffer.binding().unwrap()),
            ))
        });

        entry.dirty = false;
    }
}

pub(crate) fn prepare_cuboids_view_bind_group(
    render_device: Res<RenderDevice>,
    cuboids_pipeline: Res<CuboidsPipelines>,
    mut view_meta: ResMut<ViewMeta>,
    view_uniforms: Res<ViewUniforms>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        view_meta.cuboids_view_bind_group = Some(render_device.create_bind_group(
            "cuboids_view_bind_group",
            &cuboids_pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ));
    }
}
