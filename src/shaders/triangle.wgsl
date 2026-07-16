struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
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
    
    // Pass the texture coordinates straight through to the fragment shader
    out.tex_coords = model.tex_coords;
    
    // Multiply the vertex position by the MVP matrix to get its screen position!
    // (We add 1.0 to the position to make it a vec4)
    out.clip_position = uniforms.transform * vec4<f32>(model.position, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Read the exact pixel color from the image using the UV coordinates
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}