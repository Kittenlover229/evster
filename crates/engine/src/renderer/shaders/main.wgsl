// Groups:
// Group 0. Atlas & Palette Data: Textures, Samplers
// Group 1. Hot Data: Camera Info, Time

struct InstanceInput {
    @location(3) pos: vec2<f32>,
    @location(4) angle: f32,
    @location(5) scale: f32,
    @location(6) tint: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tint: vec3<f32>,
}

struct CameraRaw {
    view: mat4x4<f32>,
};

struct TimeRaw {
    dt: f32,
    since_start: u32,
};

@group(1) @binding(0)
var<uniform> camera: CameraRaw;
@group(1) @binding(1)
var<uniform> time: TimeRaw;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = model.tex_coords;

    let rot = mat2x2<f32>(
        vec2<f32>(cos(instance.angle), -sin(instance.angle)),
        vec2<f32>(sin(instance.angle), cos(instance.angle)),
    );

    out.clip_position = camera.view
                        * vec4<f32>(instance.pos + rot * (model.position.xy * instance.scale), model.position.z, 1.0) ;

    out.tint = instance.tint;

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(in.tint, 1.0);
}
