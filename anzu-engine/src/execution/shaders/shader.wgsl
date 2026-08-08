// Vertex Shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

struct Uniforms {
    model_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(0) @binding(1) var diffuse_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput,) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.uv = model.uv;
    out.clip_position = uniforms.model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(diffuse_texture, diffuse_sampler, in.uv);
    return texel * in.color;
}