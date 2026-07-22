use noise::{NoiseFn, Perlin};

use crate::{CHUNK_HEIGHT, CHUNK_SIZE, CHUNK_VOLUME, max_terrain_height, min_terrain_height};




pub struct MeshData {
    pub voxels: [u8; CHUNK_VOLUME],
}
impl MeshData {
    pub fn generate_mesh_data(chunk_x: i32, chunk_z: i32) -> MeshData {
        let voxels: [u8; CHUNK_VOLUME] = Self::generate_terrain(chunk_x, chunk_z);

        let mut meshdata = Self { voxels };
        meshdata
    }
    pub fn get_index(x: i32, y: i32, z: i32) -> usize {
        (x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_HEIGHT) as usize
    }
    fn generate_terrain(chunk_x: i32, chunk_z: i32) -> [u8; CHUNK_VOLUME] {
        let mut voxels = [0; CHUNK_VOLUME];
        let perlin = Perlin::new(89);

        let scale = 0.01;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = (chunk_x * CHUNK_SIZE + x) as f64;
                let world_z = (chunk_z * CHUNK_SIZE + z) as f64;

                let noise_value = perlin.get([world_x * scale, world_z * scale]);

                let normalized_noise = (noise_value + 1.0) / 2.0;

                let mut terrain_height = ((normalized_noise * CHUNK_HEIGHT as f64) as i32 - 15) / 2 ;
                if terrain_height >= max_terrain_height{
                    terrain_height = max_terrain_height;
                }
                else if terrain_height <= min_terrain_height{
                    terrain_height = min_terrain_height;
                }
                for y in 0..CHUNK_HEIGHT {
                    if y <= terrain_height {
                        let index = Self::get_index(x, y, z);
                        voxels[index] = 1;
                    } else {
                        let index = Self::get_index(x, y, z);
                        voxels[index] = 0
                    }
                }
            }
        }
        voxels
    }
}
