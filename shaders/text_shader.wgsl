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
    @location(1) texture_pos: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = uniforms.color;
    out.clip_position = uniforms.transform * vec4<f32>(model.position, 0.0, 1.0);
    out.texture_pos = (model.position + 1.0) / 2.0;
    return out;
}



@group(0)
@binding(1)
var texture: texture_2d<f32>;

@group(0)
@binding(2)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var alpha: f32 = textureSample(texture, tex_sampler, in.texture_pos).r;

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
