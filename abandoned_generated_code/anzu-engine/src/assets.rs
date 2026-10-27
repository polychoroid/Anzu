pub mod mesh_provider;

use crate::assets::mesh_provider::Primitive;
use nalgebra::Vector3;
use std::fs::File;

pub struct Mesh {
    pub id: u32,
    pub points: Vec<Vector3<f32>>,
    pub index: Option<Vec<u16>>,
    pub uv: Option<Vec<[f32; 2]>>,
}

impl Mesh {
    pub fn indices(&self) -> Option<&[u16]> {
        self.index.as_deref()
    }
}

pub enum MeshType {
    BuiltIn(Primitive),
    FromFile(File),
}
