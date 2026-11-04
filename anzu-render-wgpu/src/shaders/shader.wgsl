struct VertexInput {
 @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
};

struct Material {
    base_color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> transform_matrix: mat4x4<f32>;
@group(1) @binding(0) var<uniform> material: Material;

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = transform_matrix * vec4<f32>(vertex.position, 1.0);
    out.local_position = vertex.position;
    return out;
}

@fragment
fn fs_main(vo: VertexOutput, ) -> @location(0) vec4<f32> {
    return material.base_color;  
}