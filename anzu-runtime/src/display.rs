use std::io::Error;

use crate::assets::{AssetHandle, DrawAssetInstance};

#[derive(Debug)]
pub enum SupportedRenderBackends {
    WebGpu,
    WebGl,
    Native,
}

pub trait DisplayAdapter {
    type Target;
    type Backend: RenderBackend;

    fn attach_target(&mut self, target: Self::Target);
    fn load_assets(&mut self, assets: Vec<AssetHandle>) -> Result<(), Error>;

    fn resize(&mut self, width: u32, height: u32);
    fn render(&mut self, transforms: Vec<DrawAssetInstance>) -> Result<(), Error>;

    fn render_if_visible(
        &mut self,
        width: u32,
        height: u32,
        data: Vec<DrawAssetInstance>,
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
    fn render(&mut self, data: Vec<DrawAssetInstance>) -> Result<(), Error>;
    fn load_assets(&mut self, assets: Vec<AssetHandle>);
}
