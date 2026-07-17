use crate::mesh_data;
use crate::chunk_mesh;

pub struct Chunk{
    pub chunk_position_x: i32,
    pub chunk_positon_y: i32,
    pub chunk_position_z: i32,
    pub mesh_data: mesh_data::MeshData,
    pub chunk_mesh: chunk_mesh::ChunkMesh,
}
impl Chunk{
    pub fn new_chunk(chunk_x: i32, chunk_z: i32, device:  &wgpu::Device) -> Chunk{
        let mesh_data = mesh_data::MeshData::generate_mesh_data(chunk_x, chunk_z);
        let chunk_mesh = chunk_mesh::ChunkMesh::generate_chunk_mesh(chunk_x, chunk_z, &device, &mesh_data);

        let mut chunk = Self{
            chunk_position_x: chunk_x,
            chunk_positon_y: 0,
            chunk_position_z: chunk_z,
            mesh_data,
            chunk_mesh: chunk_mesh,

        };
        chunk
    }
}