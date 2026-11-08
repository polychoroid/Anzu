use gltf::{Document, Error, image::Format};
use log::{info, warn};
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, Extent3d, Queue, Sampler,
    TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension,
    TextureFormat::{self},
    TextureUsages, VertexAttribute, VertexBufferLayout, VertexFormat,
    util::DeviceExt,
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
    pub fn new(
        device: &Device,
        queue: &Queue,
        source: gltf::Material,
        images: &Vec<gltf::image::Data>,
    ) -> Self {
        let uniform = MaterialUniform {
            base_color: source.pbr_metallic_roughness().base_color_factor(),
        };

        let texture_details = source
            .pbr_metallic_roughness()
            .base_color_texture()
            .and_then(|info| images.get(info.texture().source().index()))
            .and_then(|texture_image_data| {
                let format = Self::color_format_map(texture_image_data.format)?;

                Some(Self::get_texture_details(
                    device,
                    texture_image_data.width,
                    texture_image_data.height,
                    texture_image_data.pixels.clone(),
                    format,
                ))
            })
            .unwrap_or_else(|| {
                Material::get_texture_details(
                    device,
                    1,
                    1,
                    vec![255, 255, 255, 255],
                    TextureFormat::Rgba8UnormSrgb,
                )
            });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material Bind Group"),
            layout: &Material::layout(&device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_details
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_details.texture_sampler),
                },
            ],
        });

        let block_size = texture_details
            .texture
            .format()
            .block_copy_size(Some(wgpu::TextureAspect::All));

        queue.write_texture(
            texture_details.texture.as_image_copy(),
            texture_details.texture_image.as_ref(),
            TexelCopyBufferLayout {
                offset: 0, // needs to be different for each unique texture
                bytes_per_row: Some(texture_details.texture.width() * block_size.unwrap()), // unwrap risks acceptable here for now
                rows_per_image: Some(texture_details.texture.height()),
            },
            texture_details.texture.size(),
        );

        Material {
            bind_group: bind_group,
            buffer: buffer,
        }
    }

    pub fn layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn get_texture_details(
        device: &Device,
        width: u32,
        height: u32,
        image: Vec<u8>,
        texture_format: TextureFormat,
    ) -> TextureDetails {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture {info.texture().source().index()}"),
            size: Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1, //future exploration
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_image = image;

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        return TextureDetails {
            texture,
            texture_image,
            texture_sampler,
        };
    }

    fn color_format_map(gltf_format: Format) -> Option<TextureFormat> {
        match gltf_format {
            Format::R8G8B8A8 => {
                info!("Texture format: {:?}", gltf_format);
                Some(TextureFormat::Rgba8UnormSrgb)
            }
            _ => {
                warn!("Unsupported texture format: {:?}", gltf_format);
                None
            }
        }
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
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub fn load_model(bytes: &[u8], device: &Device, queue: &Queue) -> Result<Asset, LoadError> {
    // We'll store the imports later so we can reuse assets without having to re-import them.
    let import = Import::new(gltf::import_slice(bytes)?);

    let materials: Vec<Material> = import
        .document
        .materials()
        .map(|material| Material::new(&device, &queue, material, &import.images))
        .collect();

    let meshes: Result<Vec<Mesh>, LoadError> = import
        .document
        .meshes()
        .map(|mesh| {
            let primitives: Result<Vec<Primitive>, LoadError> = mesh
                .primitives()
                .map(|source_primitive| {
                    let material = source_primitive.material().index();

                    let reader = source_primitive.reader(|b| Some(&import.buffers[b.index()]));

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
        meshes: meshes?,
        materials: materials,
        draws: Vec::new(),
    })
}

pub struct DrawData;

pub struct Import {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

impl Import {
    pub fn new(import: (Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>)) -> Self {
        Import {
            document: import.0,
            buffers: import.1,
            images: import.2,
        }
    }
}

pub struct Asset {
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

struct TextureDetails {
    texture: Texture,
    texture_image: Vec<u8>,
    texture_sampler: Sampler,
}
