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

struct WaterUniform {
    specular: f32,
    density: f32,
    specular_color: vec3f,
    alpha: f32,
    wave_speed: vec2f,
    wave_scale: vec2f,
    wave_height: f32,
}

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@group(0) @binding(1)
var scene_sampler: sampler;

@group(0) @binding(2)
var opaque_texture: texture_2d<f32>;

@group(0) @binding(3)
var depth_texture: texture_depth_2d;

@group(1) @binding(0)
var<uniform> water: WaterUniform;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) color: vec3f
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4f,
    @location(1) @interpolate(perspective) position: vec3f,
    @location(2) @interpolate(perspective) color: vec3f
}

fn random(v: vec2f) -> vec2f {
    let d = vec2f(
        dot(v, vec2f(127.1, 311.7)),
        dot(v, vec2f(269.5, 183.3))
    );

    return -1.0 + 2.0 * fract(sin(d) * 43758.5453123);
}

fn noise(v: vec2f) -> f32 {
    let i = floor(v);
    let f = fract(v);

    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(
        mix(
            dot(random(i + vec2f(0.0, 0.0)), f - vec2f(0.0, 0.0)),
            dot(random(i + vec2f(1.0, 0.0)), f - vec2f(1.0, 0.0)),
            u.x
        ),
        mix(
            dot(random(i + vec2f(0.0, 1.0)), f - vec2f(0.0, 1.0)),
            dot(random(i + vec2f(1.0, 1.0)), f - vec2f(1.0, 1.0)),
            u.x
        ),
        u.y
    );
}

fn calc_global_light(color: vec3f, pos: vec3f, n: vec3f) -> vec3f {
    let e = normalize(pos - scene.camera_pos);
    let r = normalize(reflect(-scene.global_light.dir, n));

    let k = scene.ambient_light +
        scene.global_light.color * max(dot(n, scene.global_light.dir), 0.0) +
        water.specular_color * pow(max(dot(e, r), 0.0), water.specular);

    return color * k;
}

fn linearize_depth(depth: f32) -> f32 {
    return
        (2.0 * scene.near_plane * scene.far_plane) /
        (scene.far_plane + scene.near_plane - depth * (scene.far_plane - scene.near_plane));
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let h = noise(
        in.position.xz * water.wave_scale + water.wave_speed * scene.time
    ) * water.wave_height;

    let p = in.position + vec3f(0.0, h, 0.0);
    let np = scene.view_proj_matrix * vec4f(p, 1.0);

    let out = VertexOutput(
        np,
        p,
        in.color
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let n = normalize(cross(
        normalize(dpdy(in.position)),
        normalize(dpdx(in.position))
    ));

    let uv = in.clip_pos.xy / scene.surface_size;
    let depth = linearize_depth(
        textureSample(depth_texture, scene_sampler, uv)
    ) / scene.far_plane;
    let curr_depth = linearize_depth(
        in.clip_pos.z
    ) / scene.far_plane;
    let dist = depth - curr_depth;

    let k = 1.0 - pow(2.0, -water.density * dist);

    return vec4f(calc_global_light(in.color, in.position, n), k);
}
