struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) @interpolate(flat) tex_index: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) tex_index: u32
}

@vertex
fn vs_main(
    model: Vertex
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    out.tex_index = model.tex_index;
    return out;
}

@group(0) @binding(0)
var textures: texture_2d_array<f32>;
@group(0) @binding(1)
var tsampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures, tsampler, in.tex_coords, in.tex_index);
}