
use wgpu::util::DeviceExt;

use crate::{CHUNK_VOLUME, Vertex, mesh_data};
pub struct ChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub chunk_x: i32,
    pub chunk_z: i32,
}
const DIRECTIONS: [[i32; 3]; 6] = [
    [1, 0, 0],  // Right: look 1 block over on the positive X axis
    [-1, 0, 0], // Left: look 1 block over on the negative X axis
    [0, 1, 0],  // Top: look 1 block up on the positive Y axis
    [0, -1, 0], // Bottom: look 1 block down on the negative Y axis
    [0, 0, 1],  // Front: look 1 block forward on the positive Z axis
    [0, 0, -1], // Back: look 1 block backward on the negative Z axis
];
impl ChunkMesh{
    pub fn get_face_vertices(face_index: usize, x: f32, y: f32, z: f32) -> [Vertex; 4] {
        let x1 = x + 1.0;
        let y1 = y + 1.0;
        let z1 = z + 1.0;

        match face_index {
            // 0: Right face (+X)
            0 => [
                Vertex {
                    position: [x1, y, z1],
                    tex_coords: [1.0, 1.0],
                }, // Bottom-Right
                Vertex {
                    position: [x1, y, z],
                    tex_coords: [0.0, 1.0],
                }, // Bottom-Left
                Vertex {
                    position: [x1, y1, z],
                    tex_coords: [0.0, 0.0],
                }, // Top-Left
                Vertex {
                    position: [x1, y1, z1],
                    tex_coords: [1.0, 0.0],
                }, // Top-Right
            ],
            // 1: Left face (-X)
            1 => [
                Vertex {
                    position: [x, y, z],
                    tex_coords: [1.0, 1.0],
                }, // Bottom-Right
                Vertex {
                    position: [x, y, z1],
                    tex_coords: [0.0, 1.0],
                }, // Bottom-Left
                Vertex {
                    position: [x, y1, z1],
                    tex_coords: [0.0, 0.0],
                }, // Top-Left
                Vertex {
                    position: [x, y1, z],
                    tex_coords: [1.0, 0.0],
                }, // Top-Right
            ],
            // 2: Top Face (+Y)
            2 => [
                Vertex {
                    position: [x1, y1, z],
                    tex_coords: [1.0, 1.0],
                }, // Bottom-Right
                Vertex {
                    position: [x, y1, z],
                    tex_coords: [0.0, 1.0],
                }, // Bottom-Left
                Vertex {
                    position: [x, y1, z1],
                    tex_coords: [0.0, 0.0],
                }, // Top-Left
                Vertex {
                    position: [x1, y1, z1],
                    tex_coords: [1.0, 0.0],
                }, // Top-Right
            ],
            // 3: Bottom Face (-Y)
            3 => [
                Vertex {
                    position: [x1, y, z1],
                    tex_coords: [1.0, 1.0],
                }, // Bottom-Right
                Vertex {
                    position: [x, y, z1],
                    tex_coords: [0.0, 1.0],
                }, // Bottom-Left
                Vertex {
                    position: [x, y, z],
                    tex_coords: [0.0, 0.0],
                }, // Top-Left
                Vertex {
                    position: [x1, y, z],
                    tex_coords: [1.0, 0.0],
                }, // Top-Right
            ],
            // 4: Front Face (+Z)
            4 => [
                Vertex {
                    position: [x, y, z1],
                    tex_coords: [1.0, 1.0],
                }, // Bottom-Right
                Vertex {
                    position: [x1, y, z1],
                    tex_coords: [0.0, 1.0],
                }, // Bottom-Left
                Vertex {
                    position: [x1, y1, z1],
                    tex_coords: [0.0, 0.0],
                }, // Top-Left
                Vertex {
                    position: [x, y1, z1],
                    tex_coords: [1.0, 0.0],
                }, // Top-Right
            ],
            // 5: Back Face (-Z)
            5 => [
                Vertex {
                    position: [x1, y, z],
                    tex_coords: [1.0, 1.0],
                }, // Bottom-Right
                Vertex {
                    position: [x, y, z],
                    tex_coords: [0.0, 1.0],
                }, // Bottom-Left
                Vertex {
                    position: [x, y1, z],
                    tex_coords: [0.0, 0.0],
                }, // Top-Left
                Vertex {
                    position: [x1, y1, z],
                    tex_coords: [1.0, 0.0],
                }, // Top-Right
            ],
            _ => panic!("Invalid face index"),
        }
    }
    pub fn get_voxel(
        x: i32,
        y: i32,
        z: i32,
        chunk_x: i32,
        chunk_z: i32,
        voxels: [u8; mesh_data::CHUNK_VOLUME],
    ) -> u8 {
        if x < 0 || x >= mesh_data::CHUNK_SIZE || y < 0 || y >= mesh_data::CHUNK_SIZE || z < 0 || z >= mesh_data::CHUNK_SIZE {
            return 0;
        }

        voxels[mesh_data::MeshData::get_index(x, y, z)]
    }
    pub fn generate_chunk_mesh(
        chunk_x: i32,
        chunk_z: i32,
        device: &wgpu::Device,
        mesh_data: &mesh_data::MeshData,
    ) -> ChunkMesh {
        let mesh_data = mesh_data::MeshData::generate_mesh_data(chunk_x, chunk_z);
        let voxels: [u8; CHUNK_VOLUME] = mesh_data.voxels;
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut vertex_count: u16 = 0;
        for z in 0..mesh_data::CHUNK_SIZE {
            for y in 0..mesh_data::CHUNK_SIZE {
                for x in 0..mesh_data::CHUNK_SIZE {
                    let current_voxel = Self::get_voxel(x, y, z, chunk_x, chunk_z, voxels);

                    if current_voxel == 0 {
                        continue;
                    }

                    for (face_index, dir) in DIRECTIONS.iter().enumerate() {
                        let neighbor_x = x + dir[0];
                        let neighbor_y = y + dir[1];
                        let neighbor_z = z + dir[2];
                        let neighbor_voxel = Self::get_voxel(
                            neighbor_x, neighbor_y, neighbor_z, chunk_x, chunk_z, voxels,
                        );

                        if neighbor_voxel == 0 {
                            let face_verts = Self::get_face_vertices(
                                face_index,
                                (x + (chunk_x * mesh_data::CHUNK_SIZE)) as f32,
                                y as f32,
                                (z + (chunk_z * mesh_data::CHUNK_SIZE)) as f32,
                            );

                            vertices.extend_from_slice(&face_verts);

                            let index_pattern = [
                                vertex_count,
                                vertex_count + 1,
                                vertex_count + 2,
                                vertex_count + 2,
                                vertex_count + 3,
                                vertex_count,
                            ];

                            indices.extend_from_slice(&index_pattern);

                            vertex_count += 4;
                        }
                    }
                }
            }
        }

        let chunk_num_indices = indices.len() as u32;

        let chunk_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let chunk_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        ChunkMesh {
            vertex_buffer: chunk_vertex_buffer,
            index_buffer: chunk_index_buffer,
            num_indices: chunk_num_indices,
            chunk_x,
            chunk_z,
        }
    }
}