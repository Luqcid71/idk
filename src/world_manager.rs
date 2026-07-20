use crate::{
    Vertex,
    chunk::{self, Chunk},
    chunk_mesh::{self, ChunkMesh},
    data_manager::{self, DataManager},
    mesh_data::{self, MeshData},
};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub const xmin: i32 = -25;
pub const xmax: i32 = 25;
pub const zmin: i32 = -25;
pub const zmax: i32 = 25;
pub struct WorldManager {
    pub chunks: HashMap<(i32, i32), Chunk>,
}
impl WorldManager {
    pub fn generate_world(device: &wgpu::Device) -> WorldManager {
        // Phase 1 (Step 1 above): parallel voxel gen
        let data_manager = data_manager::DataManager::generate_chunk_data();
        let data_manager = Arc::new(data_manager); // now cheaply shareable across threads

        let coords: Vec<(i32, i32)> = data_manager.chunk_data_dictionary.keys().copied().collect();

        // Phase 2: parallel meshing — CPU only, no wgpu::Device touched here
        let mesh_results: Vec<((i32, i32), (Vec<Vertex>, Vec<u16>))> = coords
            .par_iter()
            .map(|&(cx, cz)| {
                // Arc<DataManager> derefs to &DataManager automatically here
                let (verts, indices) = ChunkMesh::build_mesh_data(cx, cz, &data_manager);
                ((cx, cz), (verts, indices))
            })
            .collect();

        // Phase 3: sequential GPU upload — main thread only, one buffer pair per chunk
        let mut chunks = HashMap::new();
        for ((cx, cz), (vertices, indices)) in mesh_results {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            let mesh = ChunkMesh {
                vertex_buffer,
                index_buffer,
                num_indices: indices.len() as u32,
                chunk_x: cx,
                chunk_z: cz,
            };

            // was: chunks.insert((cx, cz), Chunk { chunk_mesh: mesh });
            let chunk = Chunk::new_chunk(cx, cz, device, mesh);
            chunks.insert((cx, cz), chunk);
        }

        WorldManager { chunks }
    }
}
