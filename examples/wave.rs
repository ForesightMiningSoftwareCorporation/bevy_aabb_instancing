use bevy::prelude::*;
use bevy_aabb_instancing::{Cuboid, Cuboids, VertexPullingRenderPlugin};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VertexPullingRenderPlugin)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .insert_resource(Msaa { samples: 4 })
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut instances = Vec::with_capacity(1_000_000);
    let mut total_min = Vec3::splat(f32::MAX);
    let mut total_max = Vec3::splat(f32::MIN);
    for x in 0..1000 {
        for z in 0..1000 {
            let x = x as f32 - 500.0;
            let z = z as f32 - 500.0;
            let y = 0.2 * (x * x + z * z).sqrt() * ((0.05 * x).cos() + (0.05 * z).sin());
            let c = Vec3::new(x, 0.0, z);
            let min = c - Vec3::new(0.5, y.max(0.0), 0.5);
            let max = c + Vec3::new(0.5, y.max(0.0), 0.5);
            total_min = total_min.min(min);
            total_max = total_max.max(max);
            instances.push(Cuboid::new(
                min,
                max,
                Color::hsl(y % 360.0, 1.0, 0.5).as_rgba_f32(),
            ));
        }
    }
    commands
        .spawn_bundle(SpatialBundle::default())
        .insert_bundle((
            Cuboids::new(instances),
            bevy::render::primitives::Aabb::from_min_max(total_min, total_max),
        ));

    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert_bundle(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(0.0, 100.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
        ));
}
