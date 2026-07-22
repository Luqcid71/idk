use crate::{
    Vertex,
    chunk_mesh::{self, ChunkMesh},
    mesh_data::{self, MeshData},
};
use dashmap::DashMap;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
};
pub struct ChunkManager {
    pub voxel_data: Arc<DashMap<(i32, i32), Arc<MeshData>>>,
    pending_voxels: Arc<DashMap<(i32, i32), ()>>,
    pub voxel_tx: Sender<(i32, i32)>,
    pub voxel_rx: Receiver<(i32, i32)>,
    pub pending_mesh: Arc<DashMap<(i32, i32), ()>>,
    pub mesh_tx: Sender<((i32, i32), Vec<Vertex>, Vec<u32>)>,
    pub mesh_rx: Receiver<((i32, i32), Vec<Vertex>, Vec<u32>)>,
}
impl ChunkManager {
    pub fn new() -> Self {
        let (voxel_tx, voxel_rx) = mpsc::channel();
        let (mesh_tx, mesh_rx) = mpsc::channel();

        ChunkManager {
            voxel_data: Arc::new(DashMap::new()),
            pending_voxels: Arc::new(DashMap::new()),
            voxel_tx,
            voxel_rx,
            pending_mesh: Arc::new(DashMap::new()),
            mesh_tx,
            mesh_rx,
        }
    }

    pub fn poll_voxel_completions(&self) {
        while let Ok((cx, cz)) = self.voxel_rx.try_recv() {
            // (cx,cz) just finished. This could unblock meshing for itself,
            // or for any neighbor that was waiting on (cx,cz) specifically.
            for (ncx, ncz) in [
                (cx, cz),
                (cx + 1, cz),
                (cx - 1, cz),
                (cx, cz + 1),
                (cx, cz - 1),
            ] {
                self.try_dispatch_mesh(ncx, ncz);
            }
        }
    }
    fn try_dispatch_mesh(&self, cx: i32, cz: i32) {
        if self.pending_mesh.contains_key(&(cx, cz)) {
            return;
        }

        let neighbors = [
            (cx, cz),
            (cx + 1, cz),
            (cx - 1, cz),
            (cx, cz + 1),
            (cx, cz - 1),
        ];
        if !neighbors.iter().all(|k| self.voxel_data.contains_key(k)) {
            return;
        }
        self.pending_mesh.insert((cx, cz), ());
        let voxels = self.voxel_data.clone();
        let pending = self.pending_mesh.clone();
        let tx = self.mesh_tx.clone();
        rayon::spawn(move || {
            let (verts, indices) = chunk_mesh::ChunkMesh::build_mesh_data(cx, cz, &voxels);
            pending.remove(&(cx, cz));
            let _ = tx.send(((cx, cz), verts, indices));
        });
    }
    pub fn request_chunk(&self, cx: i32, cz: i32) {
        if self.voxel_data.contains_key(&(cx, cz)) {
            return;
        }
        if self.pending_voxels.contains_key(&(cx, cz)) {
            return;
        }
        self.pending_voxels.insert((cx, cz), ());

        let voxels = self.voxel_data.clone();
        let pending = self.pending_voxels.clone();
        let tx = self.voxel_tx.clone();

        rayon::spawn(move || {
            let data = Self::generate_chunk_data(cx, cz);
            voxels.insert((cx, cz), Arc::new(data));
            pending.remove(&(cx, cz));
            let _ = tx.send((cx, cz));
        })
    }

    pub fn generate_chunk_data(cx: i32, cz: i32) -> MeshData {
        let mesh_data = MeshData::generate_mesh_data(cx, cz);
        mesh_data
    }
}
