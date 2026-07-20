use crate::{
    mesh_data::{self, MeshData},
    world_manager,
};
use rayon::prelude::*;
use std::collections::HashMap;
pub struct DataManager {
    pub chunk_data_dictionary: HashMap<(i32, i32), MeshData>,
}
impl DataManager {
    pub fn generate_chunk_data() -> DataManager {
        // 1. Build the flat list of coordinates first — rayon parallelizes over
        //    an existing collection, it doesn't parallelize nested for-loops directly.
        let coords: Vec<(i32, i32)> = (world_manager::xmin..=world_manager::xmax)
            .flat_map(|x| (world_manager::zmin..=world_manager::zmax).map(move |z| (x, z)))
            .collect();

        // 2. par_iter() instead of iter() — this single change is what tells rayon
        //    "distribute these across the thread pool" instead of running sequentially.
        let dict: HashMap<(i32, i32), MeshData> = coords
            .par_iter()
            .map(|&(x, z)| ((x, z), mesh_data::MeshData::generate_mesh_data(x, z)))
            .collect();

        Self {
            chunk_data_dictionary: dict,
        }
    }
}
