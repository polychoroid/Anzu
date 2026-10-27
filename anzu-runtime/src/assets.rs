use crate::display::RenderAssetCatalog;

pub struct AssetBank<T> {
    catalog: RenderAssetCatalog<T>,
}

impl<T> AssetBank<T> {
    pub fn new() -> Self {
        Self {
            catalog: RenderAssetCatalog::new(),
        }
    }
}
