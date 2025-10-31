// Vertex shader

@group(0) @binding(0)
var<uniform> camera: Affine2D;

struct Affine2D {
    transform: mat2x2<f32>,
    translation: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};
struct InstanceInput {
    @location(3) col0: vec2<f32>,
    @location(4) col1: vec2<f32>,
    @location(5) translation: vec2<f32>,
    @location(6) color: vec4<f32>,
};

@vertex
fn vertex(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    var transform: Affine2D;
    transform.transform = mat2x2<f32> (
        instance.col0,
        instance.col1,
    );
    transform.translation = instance.translation;
    out.clip_position = vec4<f32>(camera.transform * (transform.transform * vertex.position + transform.translation) + camera.translation, 0.0, 1.0);
    out.color = vertex.color * instance.color;
    return out;
}

// Fragment shader

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}