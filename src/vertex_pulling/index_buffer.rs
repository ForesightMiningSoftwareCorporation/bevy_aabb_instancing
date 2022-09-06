use bevy::render::{
    render_resource::{Buffer, BufferUsages, BufferVec},
    renderer::{RenderDevice, RenderQueue},
};

pub struct CuboidsIndexBuffer {
    buffer: BufferVec<u32>,
}

impl CuboidsIndexBuffer {
    pub fn new() -> Self {
        Self {
            buffer: BufferVec::new(BufferUsages::INDEX),
        }
    }

    pub fn grow_to_fit_num_cuboids(
        &mut self,
        num_cuboids: u32,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
    ) {
        /// The indices for all triangles in a cuboid mesh (given 8 corner
        /// vertices).
        ///
        /// In addition to encoding the 3-bit cube corner index, we add 2 bits
        /// to indicate which of the 3 faces is being rendered.
        #[rustfmt::skip]
        #[allow(clippy::unusual_byte_groupings)]
        const CUBE_INDICES: [u32; NUM_CUBE_INDICES_USIZE] = [
            0b00_000, 0b00_010, 0b00_001, 0b00_010, 0b00_011, 0b00_001, // face XY (0)
            0b01_101, 0b01_100, 0b01_001, 0b01_001, 0b01_100, 0b01_000, // face XZ (1)
            0b10_000, 0b10_100, 0b10_110, 0b10_000, 0b10_110, 0b10_010, // face YZ (2)
        ];

        let num_indices = num_indices_for_cuboids(num_cuboids);
        let end: u32 = self.buffer.len().try_into().unwrap();
        if end < num_indices {
            for i in end..num_indices {
                let cuboid = i / NUM_CUBE_INDICES_U32;
                let cuboid_local = i % NUM_CUBE_INDICES_U32;
                self.buffer
                    .push((cuboid << 5) + CUBE_INDICES[cuboid_local as usize]);
            }
            self.buffer.write_buffer(render_device, render_queue)
        }
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.buffer()
    }
}

// Only 3 faces are actually drawn.
const NUM_CUBE_INDICES_USIZE: usize = 3 * 3 * 2;
const NUM_CUBE_INDICES_U32: u32 = 3 * 3 * 2;

pub fn num_indices_for_cuboids(num_cuboids: u32) -> u32 {
    num_cuboids * NUM_CUBE_INDICES_U32
}
