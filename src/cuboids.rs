use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType},
};

use crate::CuboidMaterialId;

/// Value that determines the color of a [`Cuboid`] based on the associated
/// [`CuboidMaterial`](crate::CuboidMaterial).
pub type Color = u32;

/// Metadata encoded in 32 bits:
///
/// - `0x000000FF`
///     - bit 0 = 0 for visible or 1 for invisible
///     - bit 1 = 0 for non-emissive or 1 for emissive
///     - bits 2-7 = unused
/// - `0x0000FF00` = unused
/// - `0xFFFF0000` = depth bias (u16)
///   - Multiplies the depth of each cuboid vertex by `1 - bias * eps` where
///     `eps = 8e-8`. This can be used with random biases to avoid Z-fighting.
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
    pub fn new(minimum: Vec3, maximum: Vec3, color: u32) -> Self {
        assert_eq!(std::mem::size_of::<Cuboid>(), 32);
        Self {
            minimum,
            meta_bits: 0,
            maximum,
            color,
        }
    }

    #[inline]
    pub fn make_visible(&mut self) -> &mut Self {
        self.meta_bits &= !1;
        self
    }

    #[inline]
    pub fn make_invisible(&mut self) -> &mut Self {
        self.meta_bits |= 1;
        self
    }

    #[inline]
    pub fn make_emissive(&mut self) -> &mut Self {
        self.meta_bits |= 0b10;
        self
    }

    #[inline]
    pub fn make_non_emissive(&mut self) -> &mut Self {
        self.meta_bits &= !0b10;
        self
    }

    #[inline]
    pub fn set_depth_bias(&mut self, bias: u16) -> &mut Self {
        self.meta_bits &= 0x0000FFFF; // clear
        self.meta_bits |= (bias as u32) << 16; // set
        self
    }
}

/// A set of cuboids to be extracted for rendering.
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

#[derive(Clone, ShaderType)]
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

    pub fn position(&self) -> Vec3 {
        self.matrix.col(3).truncate()
    }
}

#[derive(Bundle)]
pub struct CuboidsBundle {
    pub material_id: CuboidMaterialId,
    pub cuboids: Cuboids,
    pub spatial: SpatialBundle,
}
