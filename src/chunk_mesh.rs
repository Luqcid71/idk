use wgpu::util::DeviceExt;

use crate::{CHUNK_SIZE, CHUNK_VOLUME, State, Vertex, data_manager, mesh_data, world_manager};
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

impl ChunkMesh {
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
        data_manager: &data_manager::DataManager,
    ) -> u8 {
        let out_of_chunk = x < 0
            || x >= mesh_data::CHUNK_SIZE
            || y < 0
            || y >= mesh_data::CHUNK_SIZE
            || z < 0
            || z >= mesh_data::CHUNK_SIZE;
        let is_landlocked = chunk_x != world_manager::xmax 
            && chunk_x != world_manager::xmin
            && chunk_z != world_manager::zmax
            && chunk_z != world_manager::zmin;
        if out_of_chunk && is_landlocked {
            if x < 0 {
                
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x - 1, chunk_z))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(CHUNK_SIZE - 1, y, z)]
            } else if x >= mesh_data::CHUNK_SIZE {
                
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x + 1, chunk_z))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(0, y, z)]
            }
            else if y < 0{
                return 1;
            } 
            else if y >= CHUNK_SIZE{
                return 0;
            }
            else if z < 0 {
               
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x, chunk_z - 1))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(x, y, CHUNK_SIZE - 1)]
            } else if z >= mesh_data::CHUNK_SIZE {
                
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x, chunk_z + 1))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(x, y, 0)]
            } else {
                return 1;
            }
        } else if out_of_chunk && !is_landlocked {
            if x < 0 && chunk_x-1 != world_manager::xmin - 1{
                
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x - 1, chunk_z))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(CHUNK_SIZE - 1, y, z)]
            } else if x >= mesh_data::CHUNK_SIZE && chunk_x + 1 != world_manager::xmax  + 1 {
                
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x + 1, chunk_z))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(0, y, z)]
            }
            else if y < 0{
                return 1;
            } 
            else if y >= CHUNK_SIZE{
                return 0;
            }
            else if z < 0 && chunk_z-1 != world_manager::zmin - 1 {
               
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x, chunk_z - 1))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(x, y, CHUNK_SIZE - 1)]
            } else if z >= mesh_data::CHUNK_SIZE && chunk_z + 1 != world_manager::zmax + 1 {
                
                data_manager
                    .chunk_data_dictionary
                    .get(&(chunk_x, chunk_z + 1))
                    .unwrap()
                    .voxels[mesh_data::MeshData::get_index(x, y, 0)]
            } else {
                return 1;
            }
           
        } else if !out_of_chunk && !is_landlocked {
            data_manager
                .chunk_data_dictionary
                .get(&(chunk_x, chunk_z))
                .unwrap()
                .voxels[mesh_data::MeshData::get_index(x, y, z)]
        } else {
            data_manager
                .chunk_data_dictionary
                .get(&(chunk_x, chunk_z))
                .unwrap()
                .voxels[mesh_data::MeshData::get_index(x, y, z)]
        }

        /*if x<0 && chunk_x == world_manager::xmin{
         return 0;
        }
        else if x >= mesh_data::CHUNK_SIZE as i32 && chunk_x == world_manager::xmax{
         return 0;
        }
        else if z < 0 && chunk_z == world_manager::zmin{
         return 0;
        }
        else if z >= mesh_data::CHUNK_SIZE as i32 && chunk_z == world_manager::zmax{
         return 0;
        }
        else if is_landlocked && (x < 0 || x >= mesh_data::CHUNK_SIZE || y < 0 || y >= mesh_data::CHUNK_SIZE || z < 0 || z >= mesh_data::CHUNK_SIZE) {
         return 0;
        }
        else{
         voxels[mesh_data::MeshData::get_index(x, y, z)]
        }*/
    }
    pub fn generate_chunk_mesh(
        chunk_x: i32,
        chunk_z: i32,
        device: &wgpu::Device,
        data_manager: &data_manager::DataManager,
    ) -> ChunkMesh {
        let mesh_data = data_manager
            .chunk_data_dictionary
            .get(&(chunk_x, chunk_z))
            .unwrap();
        let voxels: [u8; CHUNK_VOLUME] = mesh_data.voxels;
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut vertex_count: u16 = 0;
        for z in 0..mesh_data::CHUNK_SIZE {
            for y in 0..mesh_data::CHUNK_SIZE {
                for x in 0..mesh_data::CHUNK_SIZE {
                    let current_voxel = Self::get_voxel(x, y, z, chunk_x, chunk_z, data_manager);

                    if current_voxel == 0 {
                        continue;
                    }

                    for (face_index, dir) in DIRECTIONS.iter().enumerate() {
                        let neighbor_x = x + dir[0];
                        let neighbor_y = y + dir[1];
                        let neighbor_z = z + dir[2];
                        let neighbor_voxel = Self::get_voxel(
                            neighbor_x,
                            neighbor_y,
                            neighbor_z,
                            chunk_x,
                            chunk_z,
                            data_manager,
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
