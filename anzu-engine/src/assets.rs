pub mod mesh_provider;

use crate::assets::mesh_provider::Primitive;
use nalgebra::Vector3;
use std::fs::File;

pub struct Mesh {
    pub id: u32,
    pub points: Vec<Vector3<f32>>,
    pub index: Option<Vec<u16>>,
}

pub enum MeshType {
    BuiltIn(Primitive),
    FromFile(File),
}
