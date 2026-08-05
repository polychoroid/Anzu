use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Opaque,
    Alpha,
    Additive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialDefinition {
    pub material_id: u16,
    pub blend_mode: BlendMode,
    pub base_color_tint: [f32; 4],
    pub emissive_strength: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub specular_strength: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    pub base_color_tint: [f32; 4],
    pub pbr_scalar: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialUniformLayout {
    pub stride: u64,
    pub max_materials: u32,
}

impl MaterialUniformLayout {
    /// Creates a deterministic uniform packing layout aligned to `min_alignment`.
    pub fn with_alignment(max_materials: u32, min_alignment: u64) -> Self {
        let raw_stride = std::mem::size_of::<MaterialUniform>() as u64;
        let stride = align_to(raw_stride, min_alignment.max(1));
        Self {
            stride,
            max_materials,
        }
    }

    /// Returns the byte offset for a material index using the configured stride.
    pub fn material_offset(&self, index: u32) -> u64 {
        self.stride * index as u64
    }
}

fn align_to(value: u64, alignment: u64) -> u64 {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_layout_uses_alignment_for_stride() {
        let layout = MaterialUniformLayout::with_alignment(16, 256);

        assert_eq!(layout.stride, 256);
        assert_eq!(layout.max_materials, 16);
    }

    #[test]
    fn material_offsets_follow_stride() {
        let layout = MaterialUniformLayout::with_alignment(8, 64);

        assert_eq!(layout.material_offset(0), 0);
        assert_eq!(layout.material_offset(1), 64);
        assert_eq!(layout.material_offset(7), 448);
    }
}
