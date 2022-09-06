use bevy::prelude::*;
use bevy_aabb_instancing::{Cuboid, Cuboids, VertexPullingRenderPlugin};
use rand::Rng;
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 1 })
        .add_plugin(VertexPullingRenderPlugin { outlines: true })
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    for x_batch in 0..20 {
        for z_batch in 0..20 {
            let mut instances = Vec::with_capacity(10_000);
            for x in 0..100 {
                for z in 0..100 {
                    let x = (x_batch * 100) as f32 + x as f32 - 1000.0;
                    let z = (z_batch * 100) as f32 + z as f32 - 1000.0;
                    let d = (x * x + z * z).sqrt();
                    let amp = 0.2 * d;
                    let y = amp * ((0.05 * x).cos() * (0.05 * z).sin());
                    let c = Vec3::new(x, y, z);
                    let h = 0.01 * d;
                    let min = c - Vec3::new(0.5, h, 0.5);
                    let max = c + Vec3::new(0.5, h, 0.5);
                    let visible = rng.gen_bool(0.3);
                    let depth_jitter = rng.gen();
                    instances.push(Cuboid::new(
                        min,
                        max,
                        Color::hsl(d % 360.0, 1.0, 0.5).as_rgba_u32(),
                        visible,
                        depth_jitter,
                    ));
                }
            }
            let cuboids = Cuboids::new(instances);
            let aabb = cuboids.aabb();
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert_bundle((cuboids, aabb));
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
