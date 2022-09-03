use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType},
};
use bitvec::boxed::BitBox;

/// An axis-aligned box, extending from `minimum` to `maximum`.
#[derive(Clone, Copy, Debug, ShaderType)]
#[repr(C)]
pub struct Cuboid {
    pub minimum: Vec3,
    pub maximum: Vec3,
    pub color_rgba: u32, // Encoded from Color::as_rgba_u32
}

impl Cuboid {
    pub fn new(minimum: Vec3, maximum: Vec3, color_rgba: u32) -> Self {
        Self {
            minimum,
            maximum,
            color_rgba,
        }
    }
}

/// The set of cuboids to be extracted for rendering.
#[derive(Clone, Component, Debug, Default)]
pub struct Cuboids {
    /// Instances to be rendered. These can be masked on/off by creating an optional [`CuboidsMask`] component.
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

/// Will be rendered.
pub const VISIBLE: bool = false;

/// Won't be rendered.
pub const INVISIBLE: bool = true;

/// An optional component to accompany [`Cuboids`] and set the visibility of individual instances.
#[derive(Component)]
pub struct CuboidsMask {
    /// Parallel to the `instances` on [`Cuboids`]. Only [`VISIBLE`] bits are rendered. [`INVISIBLE`] bits are not.
    pub bitmask: BitBox,
}

impl CuboidsMask {
    pub fn new(bitmask: BitBox) -> Self {
        Self { bitmask }
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
