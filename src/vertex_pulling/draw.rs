use super::{
    cuboid_cache::CuboidBufferCache, index_buffer::CuboidsIndexBuffer, pipeline::CuboidsPipeline,
};

use bevy::{
    ecs::system::{lifetimeless::*, SystemParamItem},
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            EntityRenderCommand, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
        },
        render_resource::{BindGroup, IndexFormat, PipelineCache},
        view::ViewUniformOffset,
    },
};

pub(crate) type DrawCuboids = (
    SetCuboidsPipeline,
    SetCuboidsViewBindGroup<0>,
    SetAuxBindGroup<1>,
    SetGpuTransformBufferBindGroup<2>,
    SetGpuCuboidBuffersBindGroup<3>,
    DrawVertexPulledCuboids,
);

pub(crate) struct SetCuboidsPipeline;

impl<P: PhaseItem> RenderCommand<P> for SetCuboidsPipeline {
    type Param = (SRes<PipelineCache>, SRes<CuboidsPipeline>);

    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: &P,
        params: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (pipeline_cache, cuboids_pipeline) = params;
        if let Some(pipeline) = pipeline_cache
            .into_inner()
            .get_render_pipeline(cuboids_pipeline.pipeline_id)
        {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct ViewMeta {
    pub cuboids_view_bind_group: Option<BindGroup>,
}

pub(crate) struct SetCuboidsViewBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetCuboidsViewBindGroup<I> {
    type Param = (SRes<ViewMeta>, SQuery<Read<ViewUniformOffset>>);
    #[inline]
    fn render<'w>(
        view: Entity,
        _item: Entity,
        (view_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform_offset = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            view_meta
                .into_inner()
                .cuboids_view_bind_group
                .as_ref()
                .unwrap(),
            &[view_uniform_offset.offset],
        );

        RenderCommandResult::Success
    }
}

/// Hold the bind group for color options and clipping planes.
#[derive(Default)]
pub struct AuxiliaryMeta {
    pub bind_group: Option<BindGroup>,
}

pub(crate) struct SetAuxBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetAuxBindGroup<I> {
    type Param = (SRes<CuboidBufferCache>, SRes<AuxiliaryMeta>);

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (buffer_cache, aux_meta): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffer_cache = buffer_cache.into_inner();
        let aux_meta = aux_meta.into_inner();
        let entry = buffer_cache.entries.get(&item).unwrap();
        pass.set_bind_group(
            I,
            aux_meta.bind_group.as_ref().unwrap(),
            &[entry.color_options_index],
        );
        RenderCommandResult::Success
    }
}

#[derive(Default)]
pub struct TransformsMeta {
    pub transform_buffer_bind_group: Option<BindGroup>,
}

pub(crate) struct SetGpuTransformBufferBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetGpuTransformBufferBindGroup<I> {
    type Param = (SRes<CuboidBufferCache>, SRes<TransformsMeta>);

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (buffer_cache, transforms_meta): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let transforms_meta = transforms_meta.into_inner();
        let entry = buffer_cache.into_inner().entries.get(&item).unwrap();
        pass.set_bind_group(
            I,
            transforms_meta
                .transform_buffer_bind_group
                .as_ref()
                .unwrap(),
            &[entry.transform_index],
        );
        RenderCommandResult::Success
    }
}

pub(crate) struct SetGpuCuboidBuffersBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetGpuCuboidBuffersBindGroup<I> {
    type Param = SRes<CuboidBufferCache>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        buffer_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let entry = buffer_cache.into_inner().entries.get(&item).unwrap();
        pass.set_bind_group(I, entry.instance_buffer_bind_group.as_ref().unwrap(), &[]);
        RenderCommandResult::Success
    }
}

pub(crate) struct DrawVertexPulledCuboids;

impl EntityRenderCommand for DrawVertexPulledCuboids {
    type Param = (
        SRes<CuboidBufferCache>,
        SRes<RenderAssets<CuboidsIndexBuffer>>,
    );

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (buffer_cache, index_buffers): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        use super::index_buffer::{CUBE_INDICES, CUBE_INDICES_HANDLE};
        let entry = buffer_cache.into_inner().entries.get(&item).unwrap();
        let num_cuboids = entry.instance_buffer.get().len().try_into().unwrap();
        let index_buffer = index_buffers
            .into_inner()
            .get(&CUBE_INDICES_HANDLE.typed())
            .unwrap();
        pass.set_index_buffer(index_buffer.slice(..), 0, IndexFormat::Uint32);
        pass.draw_indexed(0..(CUBE_INDICES.len() as u32), 0, 0..num_cuboids);
        RenderCommandResult::Success
    }
}
