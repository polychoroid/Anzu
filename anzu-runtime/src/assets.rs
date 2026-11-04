#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AssetId(pub u32);

pub struct AssetHandle {
    pub id: AssetId,
    pub bytes: &'static [u8],
}

pub struct DrawAssetInstance {
    pub asset_id: AssetId,
    pub transform: [[f32; 4]; 4],
}
