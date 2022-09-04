use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::ShaderType},
};

/// An axis-aligned box, extending from `minimum` to `maximum`.
#[derive(Clone, Copy, Debug, ShaderType)]
#[repr(C)]
pub struct Cuboid {
    pub minimum: Vec3,
    /// A bitmask:
    ///
    /// 0x000000FF = 0 for visible or 1 for invisible
    /// 0x0000FF00 = 0b00XYZxyz edge visibility mask where X,Y,Z are the positive cube faces and x,y,z are the negative cube faces
    /// 0xFFFF0000 = unused
    pub mask: u32,
    pub maximum: Vec3,
    /// Encoded from `Color::as_rgba_u32`
    pub color_rgba: u32,
}

impl Cuboid {
    pub fn new(minimum: Vec3, maximum: Vec3, color_rgba: u32) -> Self {
        assert_eq!(std::mem::size_of::<Cuboid>(), 32);
        Self {
            minimum,
            mask: 0,
            maximum,
            color_rgba,
        }
    }

    pub fn new_with_visibility_masks(
        minimum: Vec3,
        maximum: Vec3,
        color_rgba: u32,
        visible: bool,
        faces_visible: [bool; 6],
    ) -> Self {
        assert_eq!(std::mem::size_of::<Cuboid>(), 32);
        let mut face_mask = 0u8;
        for i in 0..6 {
            if !faces_visible[i] {
                face_mask |= 1 << i;
            }
        }
        let mask = u32::from_le_bytes([!visible as u8, face_mask, 0, 0]);
        Self {
            minimum,
            mask,
            maximum,
            color_rgba,
        }
    }
}

/// The set of cuboids to be extracted for rendering.
#[derive(Clone, Component, Debug, Default, ShaderType)]
pub struct Cuboids {
    /// Instances to be rendered. These can be masked on/off by creating an optional [`CuboidsMask`] component.
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

/// Will be rendered.
pub const VISIBLE: bool = false;

/// Won't be rendered.
pub const INVISIBLE: bool = true;

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
