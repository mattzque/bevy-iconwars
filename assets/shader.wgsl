// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_functions

@group(2) @binding(0) var icon_sheets: texture_2d_array<f32>;
@group(2) @binding(1) var icon_sheets_sampler: sampler;

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // x, y = offset and z = rotation in radians
    @location(3) transform: vec3<f32>,
    @location(4) sheet_index: u32,
    @location(5) uv_offset: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) layer: u32,
};

/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let model = mesh2d_functions::get_model_matrix(0u);

    var position = vertex.position;
    let transform_x = vertex.transform.x;
    let transform_y = vertex.transform.y;
    let angle = vertex.transform.z;

    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    let rotated_x = (cos_angle * position.x - sin_angle * position.y);
    let rotated_y = (sin_angle * position.x + cos_angle * position.y);

    position.x = rotated_x + transform_x;
    position.y = rotated_y + transform_y;

    out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(position, 1.0));
    out.uv = ((vertex.uv * 64.0) + vertex.uv_offset) / vec2<f32>(2048.0, 2048.0);
    out.layer = vertex.sheet_index;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
   let color = textureSample(icon_sheets, icon_sheets_sampler, in.uv, in.layer);
   return vec4<f32>(1.0 - color.xyz, color.w);
}