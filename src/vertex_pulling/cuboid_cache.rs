use crate::Cuboid;

use bevy::{
    prelude::*,
    render::render_resource::{BindGroup, StorageBuffer},
    utils::HashMap,
};

#[derive(Default)]
pub(crate) struct CuboidBufferCache {
    cuboids: HashMap<Entity, CachedCuboidBuffers>,
}

pub struct CachedCuboidBuffers {
    buffers: GpuCuboidBuffers,
    position: Vec3,
    keep_alive: bool,
    enabled: bool,
}

impl CachedCuboidBuffers {
    pub fn buffers(&self) -> &GpuCuboidBuffers {
        &self.buffers
    }

    pub fn buffers_mut(&mut self) -> &mut GpuCuboidBuffers {
        &mut self.buffers
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn keep_alive(&mut self) {
        self.keep_alive = true;
    }
}

impl CuboidBufferCache {
    pub fn get(&self, entity: Entity) -> Option<&CachedCuboidBuffers> {
        self.cuboids.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut CachedCuboidBuffers> {
        self.cuboids.get_mut(&entity)
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        position: Vec3,
        enabled: bool,
        buffers: GpuCuboidBuffers,
    ) {
        self.cuboids.insert(
            entity,
            CachedCuboidBuffers {
                buffers,
                position,
                keep_alive: false,
                enabled,
            },
        );
    }

    pub fn cull_entities(&mut self) {
        let mut to_remove = Vec::new();
        for (entity, entry) in self.cuboids.iter_mut() {
            if !entry.keep_alive {
                to_remove.push(*entity);
            }
            entry.keep_alive = false;
        }
        for entity in to_remove {
            self.cuboids.remove(&entity);
        }
    }
}

#[derive(Component)]
pub struct GpuCuboidBuffers {
    pub(crate) _instance_buffer: StorageBuffer<Vec<Cuboid>>,
    pub(crate) instance_buffer_bind_group: BindGroup,
    pub(crate) color_options_index: u32,
    pub(crate) transform_index: u32,
    pub(crate) num_cuboids: u32,
}
