use bevy::{
    core::cast_slice,
    ecs::system::lifetimeless::SRes,
    prelude::HandleUntyped,
    reflect::{TypePath, TypeUuid},
    render::{
        render_asset::RenderAsset,
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::RenderDevice,
    },
};

#[derive(Default, TypeUuid, TypePath)]
#[uuid = "8f6d78a6-fffe-4e54-81db-08b0739a947a"]
pub struct CuboidsIndexBuffer;

pub(crate) const CUBE_INDICES_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(CuboidsIndexBuffer::TYPE_UUID, 17343092250772987267);

// Only 3 faces are actually drawn.
const NUM_CUBE_INDICES_USIZE: usize = 3 * 3 * 2;

/// The indices for all triangles in a cuboid mesh (given 8 corner
/// vertices).
///
/// In addition to encoding the 3-bit cube corner index, we add 2 bits
/// to indicate which of the 3 faces is being rendered.
#[rustfmt::skip]
#[allow(clippy::unusual_byte_groupings)]
pub(crate) const CUBE_INDICES: [u32; NUM_CUBE_INDICES_USIZE] = [
    0b00_000, 0b00_010, 0b00_001, 0b00_010, 0b00_011, 0b00_001, // face XY (0)
    0b01_101, 0b01_100, 0b01_001, 0b01_001, 0b01_100, 0b01_000, // face XZ (1)
    0b10_000, 0b10_100, 0b10_110, 0b10_000, 0b10_110, 0b10_010, // face YZ (2)
];

impl RenderAsset for CuboidsIndexBuffer {
    type ExtractedAsset = Self;

    type PreparedAsset = Buffer;

    type Param = SRes<RenderDevice>;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        Self
    }

    fn prepare_asset(
        _extracted_asset: Self::ExtractedAsset,
        render_device: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::INDEX,
            label: Some("Cuboid Index Buffer"),
            contents: cast_slice(CUBE_INDICES.as_slice()),
        });
        Ok(buffer)
    }
}
