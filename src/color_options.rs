use bevy::prelude::*;
use bevy::render::render_resource::{DynamicUniformBuffer, ShaderType};

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
    pub scalar_hue: ScalarHueColorOptions,
    pub color_mode: ColorMode,
}

/// Dynamic controls for coloring and visibility of scalar values encoded in
/// `cuboid.color`.
///
/// These options are only available in [`COLOR_MODE_SCALAR_HUE`].
#[derive(Clone, Debug, Default, ShaderType)]
pub struct ScalarHueColorOptions {
    /// Cuboids with `cuboid.color < min_visible_value` will be clipped.
    pub min_visible_value: f32,
    /// Cuboids with `cuboid.color > max_visible_value` will be clipped.
    pub max_visible_value: f32,
    /// Cuboid colors range from blue to red hue. Cuboids with `cuboid.color <= max_blue_value` are blue.
    pub max_blue_value: f32,
    /// Cuboid colors range from blue to red hue. Cuboids with `cuboid.color >= min_red_value` are red.
    pub min_red_value: f32,
}

/// Resource used to create and modify a set of [`ColorOptions`] that are
/// automatically synced to shader uniforms.
#[derive(Clone, Debug)]
pub struct ColorOptionsMap {
    // Consumed every frame during GPU buffering.
    options: Vec<ColorOptions>,
}

impl Default for ColorOptionsMap {
    fn default() -> Self {
        Self {
            options: vec![ColorOptions {
                color_mode: COLOR_MODE_RGB,
                scalar_hue: Default::default(), // unused
            }],
        }
    }
}

impl ColorOptionsMap {
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn get(&mut self, id: ColorOptionsId) -> &ColorOptions {
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
