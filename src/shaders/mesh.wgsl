struct GlobalLight {
    light_dir: vec3f,
    light_color: vec3f
}

struct SceneUniform {
    view_proj_matrix: mat4x4f,
    global_light: GlobalLight
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

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let out = VertexOutput(
        scene.view_proj_matrix * vec4f(in.position, 1.0),
        in.color
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
