use anzu_runtime::{
    assets::{AssetHandle, DrawAssetInstance},
    display::{DisplayAdapter, RenderBackend},
};

use std::io::Error;

pub struct CrossTermDisplayAdapter;

impl DisplayAdapter for CrossTermDisplayAdapter {
    type Target = String;

    type Backend = CrossTermBackend;

    fn attach_target(&mut self, _target: Self::Target) {
        ()
    }

    fn resize(&mut self, _width: u32, _height: u32) {
        ()
    }

    fn render(&mut self, _transform: Vec<DrawAssetInstance>) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn render_if_visible(
        &mut self,
        width: u32,
        height: u32,
        transform: Vec<DrawAssetInstance>,
    ) -> Result<(), std::io::Error> {
        if width > 0 && height > 0 {
            self.render(transform)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid scrren size.",
            ))
        }
    }

    fn load_assets(&mut self, paths: Vec<AssetHandle>) -> Result<(), Error> {
        todo!()
    }
}

pub struct CrossTermBackend {}

impl RenderBackend for CrossTermBackend {
    fn resize(&mut self, width: u32, height: u32) {
        todo!()
    }

    fn render(&mut self, transform: Vec<DrawAssetInstance>) -> Result<(), std::io::Error> {
        todo!()
    }

    fn load_assets(&mut self, assets: Vec<AssetHandle>) {
        todo!()
    }
}
