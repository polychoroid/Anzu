use std::{collections::HashMap, io::Error};

pub type RenderAssetCatalog<T> = HashMap<T, Vec<PrimitiveBinding>>;

#[derive(Debug)]
pub enum SupportedRenderBackends {
    WebGpu,
    WebGl,
    Native,
}

pub trait DisplayAdapter {
    type Target;
    type Backend;

    fn connect_target(&mut self, target: Self::Target);

    fn resize(&mut self, width: u32, height: u32);
    fn render(&mut self, data: Vec<DrawBatch>) -> Result<(), Error>;

    fn render_if_visible(
        &mut self,
        width: u32,
        height: u32,
        data: Vec<DrawBatch>,
    ) -> Result<(), Error> {
        if width > 0 && height > 0 {
            self.render(data)
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid scrren size.",
            ))
        }
    }
}

pub trait RenderBackend {
    fn resize(&mut self, width: u32, height: u32);
    fn render(&mut self, data: Vec<DrawBatch>) -> Result<(), Error>;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MeshHandle(u32);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MaterialHandle(u32);

pub struct PrimitiveBinding {
    mesh: MeshHandle,
    material: MaterialHandle,
}

impl PrimitiveBinding {
    pub fn new(mesh_handle: u32, material_handle: u32) -> Self {
        Self {
            mesh: MeshHandle(mesh_handle),
            material: MaterialHandle(material_handle),
        }
    }
}

struct InstanceData {
    model_transform: [[f32; 4]; 4],
}

pub struct DrawBatch {
    instances: Vec<InstanceData>,
    mesh: MeshHandle,
    material: MaterialHandle,
}
