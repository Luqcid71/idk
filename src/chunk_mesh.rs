use std::sync::Arc;

use dashmap::DashMap;
use wgpu::util::DeviceExt;

use crate::{
    CHUNK_HEIGHT, CHUNK_SIZE, CHUNK_VOLUME, State, Vertex, chunk, chunk_manager,
    mesh_data::{self, MeshData},
};
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
const FACE_LIGHT: [f32; 6] = [
    0.7, // 0: +X (east)
    0.7, // 1: -X (west)
    1.0, // 2: +Y (top) — brightest, "sunlight"
    0.4, // 3: -Y (bottom) — darkest, rarely seen anyway
    0.85, // 4: +Z (south)
    0.5,  // 5: -Z (north)
];
impl ChunkMesh {
    pub fn get_face_vertices(
        face_index: usize,
        x: f32,
        y: f32,
        z: f32,
        block_type: u32,
    ) -> [Vertex; 4] {
        let light = FACE_LIGHT[face_index];
        let x1 = x + 1.0;
        let y1 = y + 1.0;
        let z1 = z + 1.0;

        match face_index {
            // 0: Right face (+X)
            0 => [
                Vertex {
                    position: [x1, y, z1],
                    tex_coords: [1.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Right
                Vertex {
                    position: [x1, y, z],
                    tex_coords: [0.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Left
                Vertex {
                    position: [x1, y1, z],
                    tex_coords: [0.0, 0.0],
                    block_type,
                    light
                }, // Top-Left
                Vertex {
                    position: [x1, y1, z1],
                    tex_coords: [1.0, 0.0],
                    block_type,
                    light
                }, // Top-Right
            ],
            // 1: Left face (-X)
            1 => [
                Vertex {
                    position: [x, y, z],

                    tex_coords: [1.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Right
                Vertex {
                    position: [x, y, z1],
                    tex_coords: [0.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Left
                Vertex {
                    position: [x, y1, z1],
                    tex_coords: [0.0, 0.0],
                    block_type,
                    light
                }, // Top-Left
                Vertex {
                    position: [x, y1, z],
                    tex_coords: [1.0, 0.0],
                    block_type,
                    light
                }, // Top-Right
            ],
            // 2: Top Face (+Y)
            2 => [
                Vertex {
                    position: [x1, y1, z],
                    tex_coords: [1.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Right
                Vertex {
                    position: [x, y1, z],
                    tex_coords: [0.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Left
                Vertex {
                    position: [x, y1, z1],
                    tex_coords: [0.0, 0.0],
                    block_type,
                    light
                }, // Top-Left
                Vertex {
                    position: [x1, y1, z1],
                    tex_coords: [1.0, 0.0],
                    block_type,
                    light
                }, // Top-Right
            ],
            // 3: Bottom Face (-Y)
            3 => [
                Vertex {
                    position: [x1, y, z1],
                    tex_coords: [1.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Right
                Vertex {
                    position: [x, y, z1],
                    tex_coords: [0.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Left
                Vertex {
                    position: [x, y, z],
                    tex_coords: [0.0, 0.0],
                    block_type,
                    light
                }, // Top-Left
                Vertex {
                    position: [x1, y, z],
                    tex_coords: [1.0, 0.0],
                    block_type,
                    light
                }, // Top-Right
            ],
            // 4: Front Face (+Z)
            4 => [
                Vertex {
                    position: [x, y, z1],
                    tex_coords: [1.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Right
                Vertex {
                    position: [x1, y, z1],
                    tex_coords: [0.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Left
                Vertex {
                    position: [x1, y1, z1],
                    tex_coords: [0.0, 0.0],
                    block_type,
                    light
                }, // Top-Left
                Vertex {
                    position: [x, y1, z1],
                    tex_coords: [1.0, 0.0],
                    block_type,
                    light
                }, // Top-Right
            ],
            // 5: Back Face (-Z)
            5 => [
                Vertex {
                    position: [x1, y, z],
                    tex_coords: [1.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Right
                Vertex {
                    position: [x, y, z],
                    tex_coords: [0.0, 1.0],
                    block_type,
                    light
                }, // Bottom-Left
                Vertex {
                    position: [x, y1, z],
                    tex_coords: [0.0, 0.0],
                    block_type,
                    light
                }, // Top-Left
                Vertex {
                    position: [x1, y1, z],
                    tex_coords: [1.0, 0.0],
                    block_type,
                    light
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
        voxels: &DashMap<(i32, i32), Arc<MeshData>>,
    ) -> u8 {
        // Y has no neighbor chunks in your setup (chunks aren't stacked vertically),
        // so these two stay as absolute rules, same as your original code.
        if y < 0 {
            return 1;
        } // below world = solid (bedrock-style, matches your original)
        if y >= CHUNK_HEIGHT {
            return 0;
        } // above world = air

        // Figure out which chunk actually owns this (x,z), and the local
        // coordinate within THAT chunk. Exactly one of these four "out of range"
        // conditions can be true at a time for a given axis (assuming DIRECTIONS
        // only steps by 1, which yours does), so simple if/else-if is safe.
        let (owner_x, owner_z, local_x, local_z) = if x < 0 {
            (chunk_x - 1, chunk_z, CHUNK_SIZE - 1, z)
        } else if x >= CHUNK_SIZE {
            (chunk_x + 1, chunk_z, 0, z)
        } else if z < 0 {
            (chunk_x, chunk_z - 1, x, CHUNK_SIZE - 1)
        } else if z >= CHUNK_SIZE {
            (chunk_x, chunk_z + 1, x, 0)
        } else {
            (chunk_x, chunk_z, x, z) // normal case: inside this chunk, no neighbor lookup needed
        };

        match voxels.get(&(owner_x, owner_z)) {
            Some(data) => data.voxels[MeshData::get_index(local_x, y, local_z)],
            None => 1, // defensive fallback — see note below
        }
    }
    pub fn build_mesh_data(
        chunk_x: i32,
        chunk_z: i32,
        voxels: &DashMap<(i32, i32), Arc<MeshData>>,
    ) -> (Vec<Vertex>, Vec<u32>) {
        if !voxels.contains_key(&(chunk_x, chunk_z)) {
            return (Vec::new(), Vec::new());
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count: u32 = 0;

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for x in 0..CHUNK_SIZE {
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
                                (x + (chunk_x * CHUNK_SIZE)) as f32,
                                y as f32,
                                (z + (chunk_z * CHUNK_SIZE)) as f32,
                                current_voxel as u32,
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

        (vertices, indices)
    }
}
