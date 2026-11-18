use nalgebra::Matrix4;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AssetId(pub usize);

pub struct AssetHandle {
    pub id: AssetId,
    pub bytes: &'static [u8],
}

pub struct DrawAssetInstance {
    pub asset_id: AssetId,
    pub transform: Matrix4<f32>,
}
