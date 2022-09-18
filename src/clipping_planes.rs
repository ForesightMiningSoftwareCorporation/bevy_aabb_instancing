use bevy::{prelude::*, render::render_resource::ShaderType};

/// The range of signed distances from the plane that don't get clipped.
///
/// The plane origin and normal will be extracted from the [`GlobalTransform`],
/// assuming normal axis is pointing
#[derive(Clone, Component, Default, ShaderType)]
pub struct ClippingPlaneRange {
    /// The minimum (signed) distance from a visible cuboid's centroid to the plane.
    pub min_sdist: f32,
    /// The maximum (signed) distance from a visible cuboid's centroid to the plane.
    pub max_sdist: f32,
}

#[derive(Bundle)]
pub struct ClippingPlaneBundle {
    pub range: ClippingPlaneRange,
    #[bundle]
    pub transform: TransformBundle,
}

#[derive(Clone, Component, Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRange {
    pub origin: Vec3,
    pub unit_normal: Vec3,
    pub min_sdist: f32,
    pub max_sdist: f32,
}

#[derive(Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRanges {
    pub ranges: [GpuClippingPlaneRange; 3],
    pub num_ranges: u32,
}
