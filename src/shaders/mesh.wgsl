struct GlobalLight {
    dir: vec3f,
    color: vec3f
}

struct SceneUniform {
    view_proj_matrix: mat4x4f,
    camera_dir: vec3f,
    camera_pos: vec3f,
    surface_size: vec2f,
    near_plane: f32,
    far_plane: f32,
    global_light: GlobalLight,
    ambient_light: vec3f,
    time: f32
}

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) color: vec3f
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4f,
    @location(1) @interpolate(perspective) color: vec3f
}

fn calc_global_light(color: vec3f, n: vec3f) -> vec3f {
    let k = scene.ambient_light + scene.global_light.color * max(dot(n, scene.global_light.dir), 0.0);
    return color * k;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let out = VertexOutput(
        scene.view_proj_matrix * vec4f(in.position, 1.0),
        calc_global_light(in.color, in.normal)
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
