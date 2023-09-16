use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

use bevy_aabb_instancing::{
    COLOR_MODE_SCALAR_HUE, Cuboid, CuboidMaterial, CuboidMaterialId, CuboidMaterialMap,
    Cuboids, VertexPullingRenderPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa::Off)
        .add_plugins(VertexPullingRenderPlugin { outlines: true })
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_scalar_hue_options)
        .run();
}

fn setup(mut commands: Commands, mut material_map: ResMut<CuboidMaterialMap>) {
    let material_id = material_map.push(CuboidMaterial {
        color_mode: COLOR_MODE_SCALAR_HUE,
        ..default()
    });

    let perlin = Perlin::new(1);
    let noise_scale = 0.05;

    const PATCHES_PER_DIM: usize = 20;
    const PATCH_SIZE: usize = 10;
    const SCENE_SIZE: f32 = 500.0;
    const SCENE_SIZE_HALF: f32 = SCENE_SIZE * 0.5;
    const CELL_SIZE: f32 = SCENE_SIZE / (PATCHES_PER_DIM * PATCH_SIZE) as f32;

    for x_batch in 0..PATCHES_PER_DIM {
        for y_batch in 0..PATCHES_PER_DIM {
            for z_batch in 0..PATCHES_PER_DIM {
                let mut instances = Vec::with_capacity(PATCH_SIZE * PATCH_SIZE * PATCH_SIZE);
                for x in 0..PATCH_SIZE {
                    for y in 0..PATCH_SIZE {
                        for z in 0..PATCH_SIZE {
                            let x_pos = (x_batch * PATCH_SIZE + x) as f32 * CELL_SIZE - SCENE_SIZE_HALF;
                            let y_pos = (y_batch * PATCH_SIZE + y) as f32 * CELL_SIZE - SCENE_SIZE_HALF;
                            let z_pos = (z_batch * PATCH_SIZE + z) as f32 * CELL_SIZE - SCENE_SIZE_HALF;
                            let c = Vec3::new(x_pos, y_pos, z_pos);
                            let min = c - Vec3::new(0.5, 0.5, 0.5) * CELL_SIZE;
                            let max = c + Vec3::new(0.5, 0.5, 0.5) * CELL_SIZE;
                            
                            // Get noise value from indexes.
                            let x_f = (x_batch * PATCH_SIZE + x) as f64 * noise_scale;
                            let y_f = (y_batch * PATCH_SIZE + y) as f64 * noise_scale;
                            let z_f = (z_batch * PATCH_SIZE + z) as f64 * noise_scale;
                            let val = (perlin.get([x_f, y_f, z_f]) as f32 + 1.0).clamp(0.0, 1.0);

                            let scalar_color = u32::from_le_bytes(val.to_le_bytes());
                            let mut cuboid = Cuboid::new(min, max, scalar_color);
                            cuboid.set_depth_bias(0);
                            instances.push(cuboid);
                        }
                    }
                }
                let cuboids = Cuboids::new(instances);
                let aabb = cuboids.aabb();
                commands
                    .spawn(SpatialBundle::default())
                    .insert((cuboids, aabb, material_id));
            }
        }
    }

    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 200.0,
                ..Default::default()
            },
            Vec3::new(400.0, 20.0, 400.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
        ));
}

fn update_scalar_hue_options(time: Res<Time>, mut material_map: ResMut<CuboidMaterialMap>) {
    let material = material_map.get_mut(CuboidMaterialId(1));
    let tv = (time.elapsed_seconds() * 0.5).sin() + 1.0;
    material.scalar_hue.max_visible = tv;
    material.scalar_hue.clamp_max = 1.0;
}
