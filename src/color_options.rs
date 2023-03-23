use bevy::prelude::*;
use bevy::render::render_resource::{DynamicUniformBuffer, ShaderType};

/// Bare enum for toggling shader behavior for [`Color`].
///
/// One of:
/// - [`COLOR_MODE_RGB`]
/// - [`COLOR_MODE_SCALAR_HUE`]
pub type ColorMode = u32;

/// "Manual" coloring based on RGB-valued `cuboid.color`.
///
/// Encode with `Color::as_rgba_u32`.
pub const COLOR_MODE_RGB: ColorMode = 0;

/// "Automatic" coloring based on scalar-valued `cuboid.color`. See [`ScalarHueColorOptions`].
///
/// Encode with `u32::from_le_bytes(f32::to_le_bytes(x))`.
pub const COLOR_MODE_SCALAR_HUE: ColorMode = 1;

/// Denotes which [`ColorOptions`] to use when rendering
/// [`Cuboids`](crate::Cuboids).
///
/// When options are modified, _all_ entities with the corresponding
/// [`ColorOptionsId`] will be affected.
#[derive(Clone, Component, Copy, Eq, Hash, PartialEq)]
pub struct ColorOptionsId(pub usize);

/// Shading options, constant for each draw call.
#[derive(Clone, Debug, ShaderType)]
pub struct ColorOptions {
    pub color_mode: ColorMode,
    /// Nonzero values imply that _only_ cuboid edges will be shaded.
    /// [`VertexPullingRenderPlugin::edges`](crate::VertexPullingRenderPlugin)
    /// must be `true` for this to take effect.
    pub wireframe: u32,
    #[align(16)]
    pub scalar_hue: ScalarHueColorOptions,

    /// An extra factor that multiplies a cuboid's color when the "emissive" bit
    /// is set.
    pub emissive_gain: Vec3,
}

impl Default for ColorOptions {
    fn default() -> Self {
        Self {
            color_mode: COLOR_MODE_RGB,
            wireframe: default(),
            scalar_hue: default(),
            emissive_gain: Vec3::splat(30.0),
        }
    }
}

/// Dynamic controls for coloring and visibility of scalar values encoded in
/// `cuboid.color`.
///
/// HSL hue is determined as:
/// ```
/// // Normalize scalar value.
/// let s = (clamp(scalar, clamp_min, clamp_max) - clamp_min) / (clamp_max - clamp_min);
/// // Choose hue linearly.
/// let hue = (360.0 + hue_zero + s * hue_slope) % 360.0;
/// ```
///
/// These options are only available in [`COLOR_MODE_SCALAR_HUE`].
#[derive(Clone, Debug, ShaderType)]
pub struct ScalarHueColorOptions {
    /// Cuboids with `cuboid.color < min_visible` will be clipped.
    pub min_visible: f32,
    /// Cuboids with `cuboid.color > max_visible` will be clipped.
    pub max_visible: f32,

    pub clamp_min: f32,
    pub clamp_max: f32,
    pub hue_zero: f32,
    pub hue_slope: f32,

    pub lightness: f32,
    pub saturation: f32,
}

impl Default for ScalarHueColorOptions {
    fn default() -> Self {
        Self {
            min_visible: 0.0,
            max_visible: 1000.0,
            clamp_min: 0.0,
            clamp_max: 1000.0,
            hue_zero: 240.0,
            hue_slope: -300.0,
            lightness: 0.5,
            saturation: 1.0,
        }
    }
}

/// Resource used to create and modify a set of [`ColorOptions`] that are
/// automatically synced to shader uniforms.
#[derive(Clone, Debug, Resource)]
pub struct ColorOptionsMap {
    // Consumed every frame during GPU buffering.
    options: Vec<ColorOptions>,
}

impl Default for ColorOptionsMap {
    fn default() -> Self {
        Self {
            options: vec![default()],
        }
    }
}

impl ColorOptionsMap {
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn clear(&mut self) {
        self.options.clear();
    }

    pub fn get(&self, id: ColorOptionsId) -> &ColorOptions {
        &self.options[id.0]
    }

    pub fn get_mut(&mut self, id: ColorOptionsId) -> &mut ColorOptions {
        &mut self.options[id.0]
    }

    pub fn push(&mut self, options: ColorOptions) -> ColorOptionsId {
        let id = ColorOptionsId(self.options.len());
        self.options.push(options);
        id
    }

    pub(crate) fn write_uniforms(
        &self,
        uniforms: &mut DynamicUniformBuffer<ColorOptions>,
    ) -> Vec<ColorOptionsUniformIndex> {
        uniforms.clear();
        let mut indices = Vec::new();
        for options in self.options.iter() {
            indices.push(ColorOptionsUniformIndex(uniforms.push(options.clone())));
        }
        indices
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub(crate) struct ColorOptionsUniformIndex(pub u32);
