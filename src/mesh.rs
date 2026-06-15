use std::path::PathBuf;

use crate::math::{Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub textures: Vec<PathBuf>,
    pub normal_maps: Vec<PathBuf>,
    pub batches: Vec<DrawBatch>,
    pub has_material_library: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawBatch {
    pub first_vertex: usize,
    pub vertex_count: usize,
    pub texture: Option<usize>,
    pub normal_map: Option<usize>,
}

impl Mesh {
    pub fn triangle_count(&self) -> usize {
        self.vertices.len() / 3
    }
}
