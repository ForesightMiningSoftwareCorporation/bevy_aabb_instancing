use bevy::{prelude::*, render::render_resource::ShaderType};

/// The range of signed distances from the plane that don't get clipped.
///
/// The plane origin and normal will be extracted from the [`GlobalTransform`],
/// assuming normal axis is pointing
#[derive(Clone, Component, Debug, ShaderType)]
pub struct ClippingPlaneRange {
    /// The minimum (signed) distance from a visible cuboid's centroid to the plane.
    pub min_sdist: f32,
    /// The maximum (signed) distance from a visible cuboid's centroid to the plane.
    pub max_sdist: f32,
}

impl Default for ClippingPlaneRange {
    fn default() -> Self {
        Self {
            min_sdist: 0.0,
            max_sdist: f32::INFINITY,
        }
    }
}

#[derive(Bundle, Default)]
pub struct ClippingPlaneBundle {
    pub range: ClippingPlaneRange,
    pub transform: TransformBundle,
}

#[derive(Clone, Component, Debug, Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRange {
    pub origin: Vec3,
    pub unit_normal: Vec3,
    pub min_sdist: f32,
    pub max_sdist: f32,
}

#[derive(Debug, Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRanges {
    pub ranges: [GpuClippingPlaneRange; MAX_CLIPPING_PLANES],
    pub num_ranges: u32,
}

/// The clipping shader is `O(planes * cuboids)`, so we set a reasonable limit.
pub const MAX_CLIPPING_PLANES: usize = 16;
