pub enum EmitterType {
    Directional,
    Point,
    Spot,
    Area,
    Environment,
    Surface,
}

// pub enum MaterialTypes {
//     Wood,
//     Metal,
//     Concrete,
//     Glass,
//     Stone,
// }

pub enum Texutre {
    Brick,
    None,
}

pub enum Sampler {
    Basic,
}

pub struct Display {
    mesh_id: u32,
    material_id: u32,
    emitter_id: Option<u32>,
}

pub struct Emitter {
    id: u32,
    emitter: EmitterType,
    intensity: f32,
    color: Vec<u8>,
}

pub struct EmitterRuntime {}

pub struct EmitterBuilder {}

impl EmitterBuilder {
    // fn new(device: &wgpu::Device) -> self {}
    // fn with_layout(self, layout: wgpu::BindGroupLayout) -> self {}
    // fn with_shadow_support(self) -> self {}
    // fn build(self, spec: &LightSpec, queue: wgpu::Queue) -> Result<LightRuntime, Err> {}
    // fn update(
    //     runtime: &mut LightRuntime,
    //     spec: &LightSpec,
    //     queue: &wgpu::Queue,
    // ) -> Result<(), Err> {
    // }
}

pub struct Material {
    id: u32,
    texture_id: Option<u32>,
    sampler_id: Option<u32>,
}

pub struct MaterialRuntime {}

pub struct MaterialBuilder {}

impl MaterialBuilder {
    // fn new(device: &wgpu::Device) -> Self {}
    // fn with_pipeline_cache(self, cache: Arc<PipelineCache>) -> Self {}
    // fn with_fallbacks(self, fb: MaterliaFallbacks) -> Self {}
    // fn build(
    //     self,
    //     runtime: &mut MaterialRuntime,
    //     spec: &MaterialSpec,
    //     queue: &wgpu::Queue,
    // ) -> Result<MaterialRuntime, BuildError> {
    // }
    // fn update(
    //     runtime: &mut MaterialRuntime,
    //     spec: &MaterialSpec,
    //     queue: &wgpu::Queue,
    // ) -> Result<(), BuildError> {
    // }
}
