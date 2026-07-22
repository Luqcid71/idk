struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) block_type: u32,
    @location(3) light: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) block_type: u32,
    @location(2) light: f32,
};

// --- Bind Group 0: Camera / Transforms ---
struct Uniforms {
    transform: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// --- Bind Group 1: Texture / Sampler ---
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    
    out.tex_coords = model.tex_coords;
    out.block_type = model.block_type;
    out.light = model.light;
    out.clip_position = uniforms.transform * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Read the exact pixel color from the image using the UV coordinates
    var color: vec4<f32>;
    if in.block_type == 1u {
        color = vec4<f32>(0.1, 0.5, 0.1, 1.0);
    } else if in.block_type == 2u {
        color = vec4<f32>(0.45, 0.33, 0.19, 1.0);
    } else if in.block_type == 3u {
        color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } else if in.block_type == 0u {
        color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    } else {
        color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    return vec4<f32>(color.rgb * in.light, color.a);
}