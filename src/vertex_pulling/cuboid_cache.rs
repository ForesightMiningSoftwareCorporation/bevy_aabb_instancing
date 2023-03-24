use crate::Cuboid;

use bevy::{
    prelude::*,
    render::render_resource::{BindGroup, StorageBuffer},
    utils::HashMap,
};

#[derive(Default, Resource)]
pub(crate) struct CuboidBufferCache {
    pub entries: HashMap<Entity, CachedCuboidBuffers>,
}

#[derive(Default)]
pub(crate) struct CachedCuboidBuffers {
    pub material_index: u32,
    pub dirty: bool,
    pub enabled: bool,
    pub keep_alive: bool,
    pub instance_buffer: StorageBuffer<Vec<Cuboid>>,
    pub instance_buffer_bind_group: Option<BindGroup>,
    pub position: Vec3,
    pub transform_index: u32,
}

impl CuboidBufferCache {
    pub fn cull_entities(&mut self) {
        let mut to_remove = Vec::new();
        for (entity, entry) in self.entries.iter_mut() {
            if !entry.keep_alive {
                to_remove.push(*entity);
            }
            entry.keep_alive = false;
        }
        for entity in to_remove {
            self.entries.remove(&entity);
        }
    }
}
