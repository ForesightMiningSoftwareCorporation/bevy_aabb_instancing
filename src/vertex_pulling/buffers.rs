use crate::clipping_planes::GpuClippingPlaneRanges;
use crate::cuboids::CuboidsTransform;
use crate::ColorOptions;
use bevy::prelude::{Deref, DerefMut, Resource};
use bevy::render::render_resource::{DynamicUniformBuffer, UniformBuffer};

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct DynamicUniformBufferOfColorOptions(pub(crate) DynamicUniformBuffer<ColorOptions>);

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct DynamicUniformBufferOfCuboidTransforms(
    pub(crate) DynamicUniformBuffer<CuboidsTransform>,
);

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct UniformBufferOfGpuClippingPlaneRanges(
    pub(crate) UniformBuffer<GpuClippingPlaneRanges>,
);
