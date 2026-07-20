use crate::chunk_mesh;
use crate::mesh_data;

pub struct Chunk {
    pub chunk_position_x: i32,
    pub chunk_positon_y: i32,
    pub chunk_position_z: i32,

    pub chunk_mesh: chunk_mesh::ChunkMesh,
}
impl Chunk {
    pub fn new_chunk(
        chunk_x: i32,
        chunk_z: i32,
        device: &wgpu::Device,
        chunk_mesh: chunk_mesh::ChunkMesh,
    ) -> Chunk {
        let mut chunk = Self {
            chunk_position_x: chunk_x,
            chunk_positon_y: 0,
            chunk_position_z: chunk_z,

            chunk_mesh: chunk_mesh,
        };
        chunk
    }
}
