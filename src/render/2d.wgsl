// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};
struct InstanceInput {
    @location(3) row0: vec3<f32>,
    @location(4) row1: vec3<f32>,
    @location(5) row2: vec3<f32>,
    @location(6) color: vec4<f32>,
};

@vertex
fn vertex(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let transform = mat3x3<f32> (
        instance.row0,
        instance.row1,
        instance.row2,
    );
    out.clip_position = (transform * vec3<f32>(vertex.position.x, vertex.position.y, 1.0)).xyzz;
    out.color = vertex.color * instance.color;
    return out;
}

// Fragment shader

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

