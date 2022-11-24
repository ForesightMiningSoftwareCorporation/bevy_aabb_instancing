use std::num::{NonZeroU128, NonZeroU32};

use bevy::{
    prelude::{Color, Commands, Component, Entity, FromWorld, Image, Msaa, Query, Res, ResMut},
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_resource::{
            CompareFunction, Extent3d, FilterMode, Operations, RenderPassColorAttachment, Sampler,
            SamplerDescriptor, Texture, TextureAspect, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::TextureCache,
        view::{ExtractedWindows, ViewTarget},
    },
    utils::HashMap,
};

#[derive(Component, Clone)]
pub struct GBuffer {
    pub hiz_texture: Texture,
    pub mipmap_views_all: TextureView,
    pub mipmap_views: Vec<TextureView>,
}

pub struct GBuffers {
    buffer: HashMap<Entity, GBuffer>,
    pub sampler: Sampler,
}
impl FromWorld for GBuffers {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let sampler = world
            .resource::<RenderDevice>()
            .create_sampler(&SamplerDescriptor {
                label: Some("GBuffer Sampler"),
                mag_filter: FilterMode::Linear,
                /// How to filter the texture when it needs to be minified (made smaller)
                min_filter: FilterMode::Linear,
                /// How to filter between mip map levels
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            });
        Self {
            buffer: HashMap::default(),
            sampler,
        }
    }
}

/// For each camera, prepare the GBuffers
pub fn prepare_view_targets(
    mut commands: Commands,
    windows: Res<ExtractedWindows>,
    images: Res<RenderAssets<Image>>,
    msaa: Res<Msaa>,
    render_device: Res<RenderDevice>,
    cameras: Query<(Entity, &ExtractedCamera)>,
    mut buffers: ResMut<GBuffers>,
) {
    for (entity, camera) in &cameras {
        let gbuffer = buffers.buffer.entry(entity).or_insert_with(|| {
            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("hiz_buffer"),
                size: Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 7,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
            });
            let mipmap_views = (0..7)
                .map(|i| {
                    texture.create_view(&TextureViewDescriptor {
                        label: Some(&format!("hiz_buffer_mipmap_level {}", i)),
                        format: Some(TextureFormat::R32Float),
                        dimension: Some(TextureViewDimension::D2),
                        aspect: TextureAspect::All,
                        base_mip_level: i,
                        mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                        base_array_layer: 0,
                        array_layer_count: Some(NonZeroU32::new(1).unwrap()),
                    })
                })
                .collect();
            let mipmap_views_all = texture.create_view(&TextureViewDescriptor {
                label: Some("hiz_buffer_mipmap_all"),
                format: Some(TextureFormat::R32Float),
                dimension: Some(TextureViewDimension::D2),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(NonZeroU32::new(7).unwrap()),
                base_array_layer: 0,
                array_layer_count: Some(NonZeroU32::new(1).unwrap()),
            });
            GBuffer {
                hiz_texture: texture,
                mipmap_views,
                mipmap_views_all,
            }
        });

        commands.entity(entity).insert(gbuffer.clone());
    }
}