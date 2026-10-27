use std::collections::HashMap;

use super::{MaterialUniformLayout, PipelineBuildStart, PipelineBuilder, PipelineSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineId {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuResourceId {
    id: u32,
}

pub struct MaterialGpuState {
    pub layout: MaterialUniformLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub material_bind_groups: Vec<wgpu::BindGroup>,
    pub fallback_bind_group: wgpu::BindGroup,
}

pub struct RenderResources {
    pub shader: wgpu::ShaderModule,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub material: MaterialGpuState,
    pub pipelines: PipelineSet,
}

pub struct MaterialBindGroups {
    pub material_bind_groups: Vec<wgpu::BindGroup>,
    pub fallback_bind_group: wgpu::BindGroup,
}

pub struct RenderInstallSummary {
    pub pipeline_ids: [PipelineId; 3],
    pub material_uniform_buffer_id: GpuResourceId,
}

pub struct RenderBackendState {
    next_pipeline_id: u32,
    next_resource_id: u32,
    pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
    resources: HashMap<GpuResourceId, wgpu::Buffer>,
}

impl RenderBackendState {
    /// Creates a backend state without owning any GPU resources yet.
    pub fn new() -> Self {
        Self {
            next_pipeline_id: 0,
            next_resource_id: 0,
            pipelines: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// Starts a strict typestate pipeline build flow.
    pub fn begin_pipeline_build(label: impl Into<String>) -> PipelineBuildStart {
        PipelineBuilder::new(label)
    }

    /// Installs built resources into backend-owned maps and returns assigned ids.
    pub fn install_resources(&mut self, resources: RenderResources) -> RenderInstallSummary {
        let opaque_id = self.alloc_pipeline_id();
        let alpha_id = self.alloc_pipeline_id();
        let additive_id = self.alloc_pipeline_id();
        self.pipelines.insert(opaque_id, resources.pipelines.opaque);
        self.pipelines.insert(alpha_id, resources.pipelines.alpha);
        self.pipelines
            .insert(additive_id, resources.pipelines.additive);

        let material_uniform_buffer_id = self.alloc_resource_id();
        self.resources.insert(
            material_uniform_buffer_id,
            resources.material.uniform_buffer,
        );

        RenderInstallSummary {
            pipeline_ids: [opaque_id, alpha_id, additive_id],
            material_uniform_buffer_id,
        }
    }

    fn alloc_pipeline_id(&mut self) -> PipelineId {
        let id = PipelineId {
            id: self.next_pipeline_id,
        };
        self.next_pipeline_id = self.next_pipeline_id.saturating_add(1);
        id
    }

    fn alloc_resource_id(&mut self) -> GpuResourceId {
        let id = GpuResourceId {
            id: self.next_resource_id,
        };
        self.next_resource_id = self.next_resource_id.saturating_add(1);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_state_starts_with_zero_capacity_ids() {
        let backend = RenderBackendState::new();

        assert_eq!(backend.next_pipeline_id, 0);
        assert_eq!(backend.next_resource_id, 0);
        assert!(backend.pipelines.is_empty());
        assert!(backend.resources.is_empty());
    }
}
