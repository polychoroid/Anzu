use std::marker::PhantomData;

use anyhow::{Result, ensure};

use super::{
    BlendMode, MaterialBindGroups, MaterialGpuState, MaterialUniform, MaterialUniformLayout,
    RenderResources,
};

pub struct Start;
pub struct ShaderReady;
pub struct LayoutReady;
pub struct MaterialBufferReady;
pub struct MaterialGroupsReady;
pub struct BlendPipelinesReady;

pub struct Missing;
pub struct Present<T>(pub T);

pub struct PipelineSet {
    pub opaque: wgpu::RenderPipeline,
    pub alpha: wgpu::RenderPipeline,
    pub additive: wgpu::RenderPipeline,
}

pub struct PipelineBuilder<State, Shader, Layout, MatLayout, MatBuffer, MatGroups, Pipelines> {
    pub(crate) label: String,
    pub(crate) shader: Shader,
    pub(crate) pipeline_layout: Layout,
    pub(crate) material_layout: MatLayout,
    pub(crate) material_buffer: MatBuffer,
    pub(crate) material_groups: MatGroups,
    pub(crate) pipelines: Pipelines,
    pub(crate) _state: PhantomData<State>,
}

/// Typestate entrypoint alias for pipeline construction.
///
/// Compile-time stage gating examples:
///
/// ```compile_fail
/// use anzu_engine::execution::render::RenderBackendState;
///
/// // Fails because `build` is not available in the `Start` state.
/// let builder = RenderBackendState::begin_pipeline_build("example");
/// let _resources = builder.build();
/// ```
///
/// ```compile_fail
/// use anzu_engine::execution::render::{
///     MaterialUniformLayout,
///     Missing,
///     PipelineBuilder,
///     Present,
///     ShaderReady,
/// };
///
/// # let make_layout = || panic!("type-only doc example");
/// # let shader = make_layout();
/// # let layout = make_layout();
/// // Fails because `with_material_layout` is only available after `LayoutReady`.
/// let builder: PipelineBuilder<
///     ShaderReady,
///     Present<wgpu::ShaderModule>,
///     Missing,
///     Missing,
///     Missing,
///     Missing,
///     Missing,
/// > = PipelineBuilder::new("example").with_shader_module(shader);
/// let _ = builder.with_material_layout(MaterialUniformLayout::with_alignment(4, 256));
/// # let _ = layout;
/// ```
///
/// ```rust
/// use anzu_engine::execution::render::RenderBackendState;
///
/// // This compiles because the start alias exposes the first transition method.
/// let _builder = RenderBackendState::begin_pipeline_build("example");
/// ```
///
/// The compile-fail snippets intentionally fail at type-check time to prove
/// stage ordering is encoded in the type system.
pub type PipelineBuildStart =
    PipelineBuilder<Start, Missing, Missing, Missing, Missing, Missing, Missing>;

impl PipelineBuilder<Start, Missing, Missing, Missing, Missing, Missing, Missing> {
    /// Starts a new strict pipeline build sequence.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shader: Missing,
            pipeline_layout: Missing,
            material_layout: Missing,
            material_buffer: Missing,
            material_groups: Missing,
            pipelines: Missing,
            _state: PhantomData,
        }
    }

    /// Moves the builder into `ShaderReady` after shader creation has completed.
    pub fn with_shader_module(
        self,
        shader: wgpu::ShaderModule,
    ) -> PipelineBuilder<
        ShaderReady,
        Present<wgpu::ShaderModule>,
        Missing,
        Missing,
        Missing,
        Missing,
        Missing,
    > {
        PipelineBuilder {
            label: self.label,
            shader: Present(shader),
            pipeline_layout: Missing,
            material_layout: Missing,
            material_buffer: Missing,
            material_groups: Missing,
            pipelines: Missing,
            _state: PhantomData,
        }
    }

    /// Creates a WGSL shader module and moves the builder into `ShaderReady`.
    pub fn create_shader_module_wgsl(
        self,
        device: &wgpu::Device,
        shader_label: Option<&str>,
        shader_source_wgsl: &str,
    ) -> Result<
        PipelineBuilder<
            ShaderReady,
            Present<wgpu::ShaderModule>,
            Missing,
            Missing,
            Missing,
            Missing,
            Missing,
        >,
    > {
        ensure!(
            !shader_source_wgsl.trim().is_empty(),
            "shader source must not be empty"
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: shader_label,
            source: wgpu::ShaderSource::Wgsl(shader_source_wgsl.into()),
        });

        Ok(self.with_shader_module(shader))
    }
}

impl
    PipelineBuilder<
        ShaderReady,
        Present<wgpu::ShaderModule>,
        Missing,
        Missing,
        Missing,
        Missing,
        Missing,
    >
{
    /// Moves the builder into `LayoutReady` once pipeline layout is available.
    pub fn with_pipeline_layout(
        self,
        pipeline_layout: wgpu::PipelineLayout,
    ) -> PipelineBuilder<
        LayoutReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Missing,
        Missing,
        Missing,
        Missing,
    > {
        PipelineBuilder {
            label: self.label,
            shader: self.shader,
            pipeline_layout: Present(pipeline_layout),
            material_layout: Missing,
            material_buffer: Missing,
            material_groups: Missing,
            pipelines: Missing,
            _state: PhantomData,
        }
    }

    /// Creates a pipeline layout and moves the builder into `LayoutReady`.
    pub fn create_pipeline_layout(
        self,
        device: &wgpu::Device,
        layout_label: Option<&str>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> Result<
        PipelineBuilder<
            LayoutReady,
            Present<wgpu::ShaderModule>,
            Present<wgpu::PipelineLayout>,
            Missing,
            Missing,
            Missing,
            Missing,
        >,
    > {
        let bind_group_layout_options: Vec<Option<&wgpu::BindGroupLayout>> = bind_group_layouts
            .iter()
            .map(|layout| Some(*layout))
            .collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: layout_label,
            bind_group_layouts: &bind_group_layout_options,
            immediate_size: 0,
        });

        Ok(self.with_pipeline_layout(pipeline_layout))
    }
}

impl
    PipelineBuilder<
        LayoutReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Missing,
        Missing,
        Missing,
        Missing,
    >
{
    /// Moves the builder into `MaterialBufferReady` with deterministic material packing metadata.
    pub fn with_material_layout(
        self,
        material_layout: MaterialUniformLayout,
    ) -> PipelineBuilder<
        MaterialBufferReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Present<MaterialUniformLayout>,
        Missing,
        Missing,
        Missing,
    > {
        PipelineBuilder {
            label: self.label,
            shader: self.shader,
            pipeline_layout: self.pipeline_layout,
            material_layout: Present(material_layout),
            material_buffer: Missing,
            material_groups: Missing,
            pipelines: Missing,
            _state: PhantomData,
        }
    }

    /// Derives material uniform layout from device limits and transitions to `MaterialBufferReady`.
    pub fn derive_material_layout_from_device(
        self,
        device: &wgpu::Device,
        max_materials: u32,
    ) -> Result<
        PipelineBuilder<
            MaterialBufferReady,
            Present<wgpu::ShaderModule>,
            Present<wgpu::PipelineLayout>,
            Present<MaterialUniformLayout>,
            Missing,
            Missing,
            Missing,
        >,
    > {
        ensure!(max_materials > 0, "max_materials must be greater than zero");

        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let material_layout = MaterialUniformLayout::with_alignment(max_materials, min_alignment);
        Ok(self.with_material_layout(material_layout))
    }
}

impl
    PipelineBuilder<
        MaterialBufferReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Present<MaterialUniformLayout>,
        Missing,
        Missing,
        Missing,
    >
{
    /// Moves the builder into `MaterialGroupsReady` with all material bind groups and fallback.
    pub fn with_material_bindings(
        self,
        material_buffer: wgpu::Buffer,
        material_bind_groups: Vec<wgpu::BindGroup>,
        fallback_bind_group: wgpu::BindGroup,
    ) -> PipelineBuilder<
        MaterialGroupsReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Present<MaterialUniformLayout>,
        Present<wgpu::Buffer>,
        Present<MaterialBindGroups>,
        Missing,
    > {
        PipelineBuilder {
            label: self.label,
            shader: self.shader,
            pipeline_layout: self.pipeline_layout,
            material_layout: self.material_layout,
            material_buffer: Present(material_buffer),
            material_groups: Present(MaterialBindGroups {
                material_bind_groups,
                fallback_bind_group,
            }),
            pipelines: Missing,
            _state: PhantomData,
        }
    }

    /// Creates the material uniform buffer and per-material bind groups from the current layout.
    pub fn create_material_bindings(
        self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        buffer_label: Option<&str>,
    ) -> Result<
        PipelineBuilder<
            MaterialGroupsReady,
            Present<wgpu::ShaderModule>,
            Present<wgpu::PipelineLayout>,
            Present<MaterialUniformLayout>,
            Present<wgpu::Buffer>,
            Present<MaterialBindGroups>,
            Missing,
        >,
    > {
        let Present(material_layout) = self.material_layout;
        let raw_uniform_size = std::mem::size_of::<MaterialUniform>() as u64;
        let total_size = material_layout.stride * material_layout.max_materials as u64;
        ensure!(
            total_size > 0,
            "material uniform buffer size must be greater than zero"
        );

        let material_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label,
            size: total_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let binding_size = wgpu::BufferSize::new(raw_uniform_size)
            .ok_or_else(|| anyhow::anyhow!("invalid material uniform binding size"))?;

        let mut material_bind_groups = Vec::with_capacity(material_layout.max_materials as usize);
        for index in 0..material_layout.max_materials {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("material-bind-group"),
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &material_buffer,
                        offset: material_layout.material_offset(index),
                        size: Some(binding_size),
                    }),
                }],
            });
            material_bind_groups.push(bind_group);
        }

        let fallback_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("material-fallback-bind-group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &material_buffer,
                    offset: 0,
                    size: Some(binding_size),
                }),
            }],
        });

        Ok(self.with_material_bindings(material_buffer, material_bind_groups, fallback_bind_group))
    }
}

impl
    PipelineBuilder<
        MaterialGroupsReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Present<MaterialUniformLayout>,
        Present<wgpu::Buffer>,
        Present<MaterialBindGroups>,
        Missing,
    >
{
    /// Moves the builder into `BlendPipelinesReady` once blend pipelines are prepared.
    pub fn with_pipeline_set(
        self,
        pipelines: PipelineSet,
    ) -> PipelineBuilder<
        BlendPipelinesReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Present<MaterialUniformLayout>,
        Present<wgpu::Buffer>,
        Present<MaterialBindGroups>,
        Present<PipelineSet>,
    > {
        PipelineBuilder {
            label: self.label,
            shader: self.shader,
            pipeline_layout: self.pipeline_layout,
            material_layout: self.material_layout,
            material_buffer: self.material_buffer,
            material_groups: self.material_groups,
            pipelines: Present(pipelines),
            _state: PhantomData,
        }
    }

    /// Creates blend-specific pipelines and transitions to `BlendPipelinesReady`.
    pub fn create_blend_pipelines(
        self,
        device: &wgpu::Device,
        pipeline_label_prefix: &str,
        color_format: wgpu::TextureFormat,
        vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
        vertex_entry_point: Option<&str>,
        fragment_entry_point: Option<&str>,
    ) -> Result<
        PipelineBuilder<
            BlendPipelinesReady,
            Present<wgpu::ShaderModule>,
            Present<wgpu::PipelineLayout>,
            Present<MaterialUniformLayout>,
            Present<wgpu::Buffer>,
            Present<MaterialBindGroups>,
            Present<PipelineSet>,
        >,
    > {
        ensure!(
            !pipeline_label_prefix.trim().is_empty(),
            "pipeline label prefix must not be empty"
        );

        let Present(shader) = &self.shader;
        let Present(pipeline_layout) = &self.pipeline_layout;

        let opaque = create_pipeline_for_blend(
            device,
            &format!("{}-opaque", pipeline_label_prefix),
            pipeline_layout,
            shader,
            color_format,
            vertex_layouts,
            BlendMode::Opaque,
            vertex_entry_point,
            fragment_entry_point,
        );
        let alpha = create_pipeline_for_blend(
            device,
            &format!("{}-alpha", pipeline_label_prefix),
            pipeline_layout,
            shader,
            color_format,
            vertex_layouts,
            BlendMode::Alpha,
            vertex_entry_point,
            fragment_entry_point,
        );
        let additive = create_pipeline_for_blend(
            device,
            &format!("{}-additive", pipeline_label_prefix),
            pipeline_layout,
            shader,
            color_format,
            vertex_layouts,
            BlendMode::Additive,
            vertex_entry_point,
            fragment_entry_point,
        );

        Ok(self.with_pipeline_set(PipelineSet {
            opaque,
            alpha,
            additive,
        }))
    }
}

fn create_pipeline_for_blend(
    device: &wgpu::Device,
    label: &str,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
    blend_mode: BlendMode,
    vertex_entry_point: Option<&str>,
    fragment_entry_point: Option<&str>,
) -> wgpu::RenderPipeline {
    let blend = match blend_mode {
        BlendMode::Opaque => None,
        BlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
        BlendMode::Additive => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                operation: wgpu::BlendOperation::Add,
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
            },
            alpha: wgpu::BlendComponent {
                operation: wgpu::BlendOperation::Add,
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
            },
        }),
    };

    let vertex_layout_options: Vec<Option<wgpu::VertexBufferLayout<'_>>> =
        vertex_layouts.iter().cloned().map(Some).collect();

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: vertex_entry_point,
            buffers: &vertex_layout_options,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: fragment_entry_point,
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

impl
    PipelineBuilder<
        BlendPipelinesReady,
        Present<wgpu::ShaderModule>,
        Present<wgpu::PipelineLayout>,
        Present<MaterialUniformLayout>,
        Present<wgpu::Buffer>,
        Present<MaterialBindGroups>,
        Present<PipelineSet>,
    >
{
    /// Finalizes the builder and returns immutable render resources.
    pub fn build(self) -> RenderResources {
        let Present(shader) = self.shader;
        let Present(pipeline_layout) = self.pipeline_layout;
        let Present(material_layout) = self.material_layout;
        let Present(uniform_buffer) = self.material_buffer;
        let Present(material_groups) = self.material_groups;
        let Present(pipelines) = self.pipelines;

        RenderResources {
            shader,
            pipeline_layout,
            material: MaterialGpuState {
                layout: material_layout,
                uniform_buffer,
                material_bind_groups: material_groups.material_bind_groups,
                fallback_bind_group: material_groups.fallback_bind_group,
            },
            pipelines,
        }
    }
}
