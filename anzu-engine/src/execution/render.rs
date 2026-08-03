use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineId {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuResourceId {
    id: u32,
}

pub struct RenderBackendState {
    pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
    resources: HashMap<GpuResourceId, wgpu::Buffer>,
}

impl RenderBackendState {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            resources: HashMap::new(),
        }
    }
}
