use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType},
};

/// An axis-aligned box, extending from `minimum` to `maximum`.
#[derive(Clone, Copy, Debug, ShaderType)]
#[repr(C)]
pub struct Cuboid {
    pub minimum: Vec3,
    /// Metadata encoded in 32 bits:
    ///
    /// - 0x000000FF = 0 for visible or 1 for invisible
    /// - 0x0000FF00 = depth jitter (u8)
    /// - 0xFFFF0000 = unused
    pub meta_bits: u32,
    pub maximum: Vec3,
    /// Encoded from `Color::as_rgba_u32`
    pub color_rgba: u32,
}

impl Cuboid {
    pub fn new(
        minimum: Vec3,
        maximum: Vec3,
        color_rgba: u32,
        visible: bool,
        depth_jitter: u8,
    ) -> Self {
        assert_eq!(std::mem::size_of::<Cuboid>(), 32);
        let mut meta_bits = (!visible) as u32;
        meta_bits |= (depth_jitter as u32) << 8;
        Self {
            minimum,
            meta_bits,
            maximum,
            color_rgba,
        }
    }
}

/// The set of cuboids to be extracted for rendering.
#[derive(Clone, Component, Debug, Default, ShaderType)]
pub struct Cuboids {
    /// Instances to be rendered.
    #[size(runtime)]
    pub instances: Vec<Cuboid>,
}

impl Cuboids {
    pub fn new(instances: Vec<Cuboid>) -> Self {
        Self { instances }
    }

    /// Automatically creates an [`Aabb`] that bounds all `instances`.
    pub fn aabb(&self) -> Aabb {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for i in self.instances.iter() {
            min = min.min(i.minimum);
            max = max.max(i.maximum);
        }
        Aabb::from_min_max(min, max)
    }
}

/// The range of signed distances from the plane that don't get clipped.
#[derive(Clone, Component, Default, ShaderType)]
pub struct ClippingPlaneRange {
    /// The minimum (signed) distance from a visible cuboid's centroid to the plane.
    pub min_sdist: f32,
    /// The maximum (signed) distance from a visible cuboid's centroid to the plane.
    pub max_sdist: f32,
}

#[derive(Bundle)]
pub struct ClippingPlaneBundle {
    pub global_transform: GlobalTransform,
    pub range: ClippingPlaneRange,
    pub transform: Transform,
}
