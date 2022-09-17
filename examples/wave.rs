use bevy::prelude::*;
use bevy_aabb_instancing::{
    ColorOptions, ColorOptionsId, ColorOptionsMap, Cuboid, Cuboids, ScalarHueColorOptions,
    VertexPullingRenderPlugin, COLOR_MODE_SCALAR_HUE,
};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 1 })
        .add_plugin(VertexPullingRenderPlugin { outlines: true })
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(setup)
        .add_system(update_scalar_hue_options)
        .run();
}

fn setup(mut commands: Commands, mut color_options_map: ResMut<ColorOptionsMap>) {
    let color_options_id = color_options_map.push(ColorOptions {
        scalar_hue: ScalarHueColorOptions {
            min_visible: 0.0,
            max_visible: 1000.0,
            clamp_min: 0.0,
            clamp_max: 1000.0,
            hue_zero: 240.0,
            hue_slope: -300.0,
        },
        color_mode: COLOR_MODE_SCALAR_HUE,
    });

    const PATCHES_PER_DIM: usize = 20;
    const PATCH_SIZE: usize = 150;
    const SCENE_RADIUS: f32 = 1500.0;

    for x_batch in 0..PATCHES_PER_DIM {
        for z_batch in 0..PATCHES_PER_DIM {
            let mut instances = Vec::with_capacity(PATCH_SIZE * PATCH_SIZE);
            for x in 0..PATCH_SIZE {
                for z in 0..PATCH_SIZE {
                    let x = (x_batch * PATCH_SIZE) as f32 + x as f32 - SCENE_RADIUS;
                    let z = (z_batch * PATCH_SIZE) as f32 + z as f32 - SCENE_RADIUS;
                    let d = (x * x + z * z).sqrt();
                    let amp = 0.2 * d;
                    let y = amp * ((0.05 * x).cos() * (0.05 * z).sin());
                    let c = Vec3::new(x, y, z);
                    let h = 0.01 * d;
                    let min = c - Vec3::new(0.5, h, 0.5);
                    let max = c + Vec3::new(0.5, h, 0.5);
                    let visible = true;
                    let depth_jitter = 0;
                    let scalar_color = u32::from_le_bytes(d.to_le_bytes());
                    instances.push(Cuboid::new(min, max, scalar_color, visible, depth_jitter));
                }
            }
            let cuboids = Cuboids::new(instances);
            let aabb = cuboids.aabb();
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert_bundle((cuboids, aabb, color_options_id));
        }
    }

    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert_bundle(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 2.0,
                ..Default::default()
            },
            Vec3::new(0.0, 100.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
        ));
}

fn update_scalar_hue_options(time: Res<Time>, mut color_options_map: ResMut<ColorOptionsMap>) {
    let options = color_options_map.get_mut(ColorOptionsId(1));
    let tv = 1000.0 * (time.seconds_since_startup().sin() + 1.0) as f32;
    options.scalar_hue.max_visible = tv;
    options.scalar_hue.clamp_max = tv;
}
