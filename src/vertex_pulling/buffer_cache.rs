use crate::component::*;
use crate::SmallKeyHashMap;

use bevy::{
    prelude::*,
    render::primitives::Aabb,
    render::render_resource::{BindGroup, Buffer},
};
use bytemuck::{Pod, Zeroable};

#[derive(Default)]
pub(crate) struct BufferCache {
    entries: SmallKeyHashMap<Entity, BufferCacheEntry>,
    // TODO: move into separate TransformMeta resource?
    pub transform_buffer_bind_group: Option<BindGroup>,
}

pub struct BufferCacheEntry {
    buffers: GpuCuboidBuffers,
    aabb: Aabb,
    keep_alive: bool,
    enabled: bool,
}

impl BufferCacheEntry {
    pub fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    pub fn buffers(&self) -> &GpuCuboidBuffers {
        &self.buffers
    }

    pub fn buffers_mut(&mut self) -> &mut GpuCuboidBuffers {
        &mut self.buffers
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

impl BufferCache {
    pub fn transform_buffer_bind_group(&self) -> Option<&BindGroup> {
        self.transform_buffer_bind_group.as_ref()
    }

    pub fn get(&self, entity: Entity) -> Option<&BufferCacheEntry> {
        self.entries.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut BufferCacheEntry> {
        self.entries.get_mut(&entity)
    }

    pub fn insert(&mut self, entity: Entity, aabb: Aabb, enabled: bool, buffers: GpuCuboidBuffers) {
        self.entries.insert(
            entity,
            BufferCacheEntry {
                buffers,
                aabb,
                keep_alive: false,
                enabled,
            },
        );
    }

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

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct GpuCuboid {
    pub min: Vec3,
    _pad0: f32,
    pub max: Vec3,
    _pad1: f32,
    pub color: [f32; 4],
}

impl From<&Cuboid> for GpuCuboid {
    fn from(cuboid: &Cuboid) -> Self {
        Self {
            min: cuboid.minimum,
            _pad0: 0.0,
            max: cuboid.maximum,
            _pad1: 0.0,
            color: cuboid.color_rgba,
        }
    }
}

#[derive(Clone, Component)]
pub struct GpuCuboidBuffers {
    pub(crate) _instance_buffer: Buffer,
    pub(crate) instance_buffer_bind_group: BindGroup,
    pub(crate) transform_index: u32,
    pub(crate) num_cuboids: u32,
}
