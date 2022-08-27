use bevy::prelude::*;
use bevy_aabb_instancing::{Cuboid, Cuboids, VertexPullingRenderPlugin};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(VertexPullingRenderPlugin)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    for x_batch in 0..30 {
        for z_batch in 0..30 {
            let mut batch_min = Vec3::splat(f32::MAX);
            let mut batch_max = Vec3::splat(f32::MIN);
            let mut instances = Vec::with_capacity(10_000);
            for x in 0..100 {
                for z in 0..100 {
                    let x = (x_batch * 100) as f32 + x as f32 - 1500.0;
                    let z = (z_batch * 100) as f32 + z as f32 - 1500.0;
                    let d = (x * x + z * z).sqrt();
                    let amp = 0.2 * d;
                    let y = amp * ((0.05 * x).cos() * (0.05 * z).sin());
                    let c = Vec3::new(x, y, z);
                    let h = 0.01 * d;
                    let min = c - Vec3::new(0.5, h, 0.5);
                    let max = c + Vec3::new(0.5, h, 0.5);
                    batch_min = batch_min.min(min);
                    batch_max = batch_max.max(max);
                    instances.push(Cuboid::new(
                        min,
                        max,
                        Color::hsl(y.abs() % 360.0, 1.0, 0.5).as_rgba_u32(),
                    ));
                }
            }
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert_bundle((
                    Cuboids::new(instances),
                    bevy::render::primitives::Aabb::from_min_max(batch_min, batch_max),
                ));
        }
    }

    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert_bundle(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(0.0, 100.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
        ));
}
