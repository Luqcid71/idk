use std::collections::HashMap;

use crate::{
    mesh_data::{self, MeshData},
    world_manager,
};
pub struct DataManager {
    pub chunk_data_dictionary: HashMap<(i32, i32), MeshData>,
}
impl DataManager {
    pub fn generate_chunk_data() -> DataManager {
        let mut dict = HashMap::new();
        for x in world_manager::xmin..=world_manager::xmax {
            for z in world_manager::zmin..=world_manager::zmax {
                dict.insert((x, z), mesh_data::MeshData::generate_mesh_data(x, z));
                println!("generated data for chunk: ({}, {})", x, z)
            }
        }
        let mut manager = Self {
            chunk_data_dictionary: dict,
        };
        manager
    }
}
