use gltf::Error;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, VertexAttribute, VertexBufferLayout,
    VertexFormat, util::DeviceExt,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

#[derive(Debug)]
pub enum LoadError {
    Gtlf(gltf::Error),
    NoPosition,
    NoScene,
    UvMisalignment,
}

impl From<gltf::Error> for LoadError {
    fn from(e: gltf::Error) -> Self {
        LoadError::Gtlf(e)
    }
}

impl From<LoadError> for std::io::Error {
    fn from(_e: LoadError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Error loading model")
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

pub struct Material {
    pub bind_group: BindGroup,
    pub buffer: Buffer,
}

impl Material {
    pub fn new(device: &Device, source: gltf::Material) -> Self {
        let uniform = MaterialUniform {
            base_color: source.pbr_metallic_roughness().base_color_factor(),
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material Bind Group"),
            layout: &Material::layout(&device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Material {
            bind_group: bind_group,
            buffer: buffer,
        }
    }

    pub fn layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

pub struct Primitive {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub material: Option<usize>,
}

impl Primitive {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            }],
        }
    }
}

pub fn load_model(bytes: &[u8], device: &Device) -> Result<Asset<Uploaded>, LoadError> {
    let (document, buffers, _images) = gltf::import_slice(bytes)?;

    let materials: Vec<Material> = document
        .materials()
        .map(|material| Material::new(&device, material))
        .collect();

    let meshes: Result<Vec<Mesh>, LoadError> = document
        .meshes()
        .map(|mesh| {
            let primitives: Result<Vec<Primitive>, LoadError> = mesh
                .primitives()
                .map(|source_primitive| {
                    let material = source_primitive.material().index();

                    let reader = source_primitive.reader(|b| Some(&buffers[b.index()]));

                    let positions = reader.read_positions().ok_or(Error::MissingBlob)?;
                    let uvs = reader
                        .read_tex_coords(0)
                        .ok_or(Error::MissingBlob)?
                        .into_f32();

                    if positions.len() != uvs.len() {
                        return Err(LoadError::UvMisalignment);
                    }

                    let verticies: Vec<Vertex> = positions
                        .zip(uvs)
                        .map(|(position, uv)| Vertex {
                            position: position,
                            uv: uv,
                        })
                        .collect();

                    let indices: Vec<u32> = reader
                        .read_indices()
                        .ok_or(Error::MissingBlob)?
                        .into_u32()
                        .collect();

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("{path} vertex buffer"),
                            contents: bytemuck::cast_slice(&verticies),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    let index_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("{path} index buffer"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                    Ok(Primitive {
                        vertex_buffer: vertex_buffer,
                        index_buffer: index_buffer,
                        index_count: indices.len() as u32,
                        material: material,
                    })
                })
                .collect();

            Ok(Mesh {
                primitives: primitives?,
            })
        })
        .collect();

    Ok(Asset {
        state: Uploaded {
            meshes: meshes?,
            materials: materials,
            draws: Vec::new(),
        },
    })
}

pub struct DrawData;

pub struct Asset<State> {
    pub state: State,
}

pub struct Imported {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub struct Uploaded {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub draws: Vec<DrawData>,
}

pub enum UploadPolicy {
    Eager,
    OnDemand,
    Never,
}

pub enum Residency {
    CpuOnly,
    UploadQueued,
    Resident,
    Evicted,
}
