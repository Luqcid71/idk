use std::collections::HashMap;

use crate::{
    chunk::{self, Chunk},
    chunk_mesh::{self, ChunkMesh},
    data_manager::{self, DataManager},
    mesh_data::{self, MeshData},
};

pub const xmin: i32 = -10;
pub const xmax: i32 = 10;
pub const zmin: i32 = -10;
pub const zmax: i32 = 10;
pub struct WorldManager {
    pub chunks: HashMap<(i32, i32), Chunk>,
}
impl WorldManager {
    pub fn generate_world(device: &wgpu::Device) -> WorldManager {
        let mut chunks = HashMap::new();
        let data = data_manager::DataManager::generate_chunk_data();

        for x in xmin..=xmax{
            for z in zmin..=zmax {
                let chunk_mesh = chunk_mesh::ChunkMesh::generate_chunk_mesh(x, z, device, &data);
                let chunk = chunk::Chunk::new_chunk(x, z, &device, chunk_mesh);
                chunks.insert((x, z), chunk);
            }
        }

        let mut manager = Self { chunks };
        manager
    }
}
