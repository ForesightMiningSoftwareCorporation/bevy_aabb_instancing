use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType},
};

pub type Color = u32;

/// Metadata encoded in 32 bits:
///
/// - `0x000000FF` = 0 for visible or 1 for invisible
/// - `0x0000FF00` = depth bias (u8)
///   - Multiplies the depth of each cuboid vertex by `1 + bias * eps` where
///     `eps = 4e-6`. This can be used with random biases to avoid Z-fighting.
/// - `0xFFFF0000` = unused
pub type MetaBits = u32;

/// An axis-aligned box, extending from `minimum` to `maximum`.
#[derive(Clone, Copy, Debug, ShaderType)]
#[repr(C)]
pub struct Cuboid {
    pub minimum: Vec3,
    pub meta_bits: MetaBits,
    pub maximum: Vec3,
    pub color: Color,
}

impl Cuboid {
    pub fn new(minimum: Vec3, maximum: Vec3, color: u32, visible: bool, depth_bias: u8) -> Self {
        assert_eq!(std::mem::size_of::<Cuboid>(), 32);
        let mut meta_bits = (!visible) as u32;
        meta_bits |= (depth_bias as u32) << 8;
        Self {
            minimum,
            meta_bits,
            maximum,
            color,
        }
    }

    #[inline]
    pub fn make_visible(&mut self) {
        self.meta_bits &= !1;
    }

    #[inline]
    pub fn make_invisible(&mut self) {
        self.meta_bits |= 1;
    }
}

/// The set of cuboids to be extracted for rendering.
#[derive(Clone, Component, Debug, Default)]
pub struct Cuboids {
    /// Instances to be rendered.
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
