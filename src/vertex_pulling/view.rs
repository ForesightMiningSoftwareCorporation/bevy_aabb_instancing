use bevy::{
    prelude::{Commands, Component, Entity, Image, Msaa, Query, Res, ResMut, Color},
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureView, RenderPassColorAttachment, Operations,
        },
        renderer::RenderDevice,
        texture::TextureCache,
        view::{ExtractedWindows, ViewTarget},
    },
    utils::HashMap,
};

#[derive(Component)]
pub struct GBuffer {
    pub entity_instance_id: TextureView,
}

impl GBuffer {
    pub fn get_color_attachment(&self, ops: Operations<Color>) -> RenderPassColorAttachment {
        use bevy::render::render_resource::LoadOp;
        RenderPassColorAttachment {
            view: &self.entity_instance_id,
            resolve_target: None,
            ops: Operations {
                load: match ops.load {
                    LoadOp::Clear(color) => LoadOp::Clear(color.into()),
                    LoadOp::Load => LoadOp::Load,
                },
                store: ops.store
            },
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
    mut texture_cache: ResMut<TextureCache>,
    cameras: Query<(Entity, &ExtractedCamera)>,
) {
    let mut sampled_textures = HashMap::default();
    for (entity, camera) in &cameras {
        if let Some(target_size) = camera.physical_target_size {
            let sampled_texture = sampled_textures
                .entry(camera.target.clone())
                .or_insert_with(|| {
                    texture_cache.get(
                        &render_device,
                        TextureDescriptor {
                            label: Some("g_buffer_entity_instance_id_attachment"),
                            size: Extent3d {
                                width: target_size.x,
                                height: target_size.y,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: TextureDimension::D2,
                            format: TextureFormat::Rg32Uint,
                            usage: TextureUsages::RENDER_ATTACHMENT,
                        },
                    )
                });

            commands.entity(entity).insert(GBuffer {
                entity_instance_id: sampled_texture.default_view.clone(),
            });
        }
    }
}
