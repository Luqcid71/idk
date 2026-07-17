use std::collections::HashMap;

use crate::chunk::{self, Chunk};


pub const xmin: i32 = -25;
pub const xmax: i32 = 25;
pub const zmin: i32 = -25;
pub const zmax: i32 = 25;
pub struct WorldManager{
    pub chunks: HashMap<(i32, i32), Chunk>,
}
impl WorldManager{
    pub fn generate_world(device: &wgpu::Device) -> WorldManager{
        let mut chunks = HashMap::new();

        for x in xmin..xmax {
            for z in zmin..zmax {
                let chunk1 = chunk::Chunk::new_chunk(x, z, &device);

                chunks.insert((x, z), chunk1);
            }
        }

    
    let mut manager = Self{
        chunks,
    };
    manager
}
}