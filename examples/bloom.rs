use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use smooth_bevy_cameras::{controllers::fps::*, LookTransformPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        .add_plugin(VertexPullingRenderPlugin { outlines: true })
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let colors = [
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::YELLOW,
        Color::PURPLE,
    ];

    let mut cuboids = Vec::new();
    for x in 0..10 {
        for y in 0..10 {
            let min = Vec3::new(x as f32 - 5.0, 0.0, y as f32 - 5.0);
            let max = min + Vec3::ONE;
            let color = colors[(x + y) % colors.len()].as_rgba_u32();
            let mut cuboid = Cuboid::new(min, max, color);
            if min.length() < 3.0 {
                cuboid.make_emissive();
            }
            cuboids.push(cuboid);
        }
    }

    let cuboids = Cuboids::new(cuboids);
    let aabb = cuboids.aabb();
    commands
        .spawn(SpatialBundle::default())
        .insert((cuboids, aabb, CuboidMaterialId(0)));

    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                tonemapping: Tonemapping::TonyMcMapface,
                ..default()
            },
            BloomSettings {
                intensity: 0.2,
                high_pass_frequency: 1.0,
                low_frequency_boost: 0.8,
                low_frequency_boost_curvature: 0.7,
                prefilter_settings: BloomPrefilterSettings {
                    threshold: 0.0,
                    threshold_softness: 0.0,
                },
                composite_mode: BloomCompositeMode::EnergyConserving,
            },
        ))
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 10.0,
                ..Default::default()
            },
            Vec3::splat(10.0),
            Vec3::ZERO,
            Vec3::Y,
        ));
}
