struct SkyboxUniform {
    transform_matrix: mat4x4f,
    sky_color: vec3f,
    horizon_color: vec3f,
    bottom_color: vec3f,
    scattering: f32
}

@group(1) @binding(0)
var<uniform> skybox: SkyboxUniform;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) color: vec3f
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4f,
    @location(0) @interpolate(perspective) position: vec3f
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let out = VertexOutput(
        skybox.transform_matrix * vec4f(in.position, 1.0),
        in.position
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let a = normalize(in.position).y;
    let k = pow(abs(a), skybox.scattering);
    let c = select(
        mix(skybox.horizon_color, skybox.bottom_color, k),
        mix(skybox.horizon_color, skybox.sky_color, k),
        a > 0.0
    );

    return vec4f(c, 1.0);
}
