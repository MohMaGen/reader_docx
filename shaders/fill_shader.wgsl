struct Uniforms {
    transform: mat4x4<f32>,
    color: vec4<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = uniforms.color;
    out.clip_position = uniforms.transform * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

 

 
