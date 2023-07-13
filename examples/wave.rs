use bevy::prelude::*;
use bevy_aabb_instancing::{
    Cuboid, CuboidMaterial, CuboidMaterialId, CuboidMaterialMap, Cuboids,
    VertexPullingRenderPlugin, COLOR_MODE_SCALAR_HUE,
};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa::Off)
        .add_plugins((
            VertexPullingRenderPlugin { outlines: true },
            LookTransformPlugin,
            FpsCameraPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_scalar_hue_options)
        .run();
}

fn setup(mut commands: Commands, mut material_map: ResMut<CuboidMaterialMap>) {
    let material_id = material_map.push(CuboidMaterial {
        color_mode: COLOR_MODE_SCALAR_HUE,
        ..default()
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
                    let scalar_color = u32::from_le_bytes(d.to_le_bytes());
                    let mut cuboid = Cuboid::new(min, max, scalar_color);
                    cuboid.set_depth_bias(0);
                    instances.push(cuboid);
                }
            }
            let cuboids = Cuboids::new(instances);
            let aabb = cuboids.aabb();
            commands
                .spawn(SpatialBundle::default())
                .insert((cuboids, aabb, material_id));
        }
    }

    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 200.0,
                ..Default::default()
            },
            Vec3::new(0.0, 100.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            Vec3::Y,
        ));
}

fn update_scalar_hue_options(time: Res<Time>, mut material_map: ResMut<CuboidMaterialMap>) {
    let material = material_map.get_mut(CuboidMaterialId(1));
    let tv = 1000.0 * (time.elapsed_seconds().sin() + 1.0);
    material.scalar_hue.max_visible = tv;
    material.scalar_hue.clamp_max = tv;
}
