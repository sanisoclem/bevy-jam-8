import bevy_pbr::render::mesh_view_bindings::{view, globals};
import bevy_pbr::render::view_transformations::position_world_to_clip;

struct Snowfall {
    strength: array<vec4<f32>, 12>,
    first_sample_age: f32,
    sample_step: f32,
    flash: f32,
    _webgl2_padding: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> snowfall: Snowfall;

struct Vertex {
    @location(0) origin: vec3<f32>,
    @location(2) corner: vec2<f32>,
    @location(3) seed: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) corner: vec2<f32>,
    @location(1) fade: f32,
}

fn swayed(pos_flake: vec3<f32>, seed: f32) -> vec3<f32> {
    let wave = pos_flake.y * 1.3 + seed * 40.0;
    return pos_flake + vec3<f32>(sin(wave), 0.0, cos(wave * 0.7)) * 0.3;
}

fn point_at_height(pos_center: vec3<f32>, pos_fall: vec3<f32>, height: f32) -> vec3<f32> {
    return pos_center + pos_fall * (height - pos_center.y) / pos_fall.y;
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let pos_field = vec3<f32>(90.0, 13.0, 90.0);
    let pos_fall = vec3<f32>(0.3, -1.5, -0.15);
    let flake_size = vec2<f32>(0.01, 0.028);
    let nearest = 0.8;
    let far_fade = vec2<f32>(32.0, 43.0);
    let rim_radius = vec2<f32>(0.7, 0.8);
    let rim_height = 1.026;
    //let t_1 = sin(global.time * 3.0);
    let t_1 = 0.6;

    var retval: VertexOutput;
    retval.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    retval.corner = in.corner;
    retval.fade = 0.0;

    let lean = fract(in.origin.xz * 41.7) - 0.5;
    let pos_drift = vec3<f32>(lean.x, 0.0, lean.y) * 0.5;
    let pos_flake_fall = (pos_fall + pos_drift) * mix(0.8, 1.2, in.seed.y);

    let pos_eye = view.world_position;
    let pos_travel = pos_flake_fall * globals.time - vec3<f32>(pos_eye.x, 0.0, pos_eye.z);
    let pos_wrapped = fract(in.origin + pos_travel / pos_field);
    let pos_center = vec3<f32>(
        pos_eye.x + (pos_wrapped.x - 0.5) * pos_field.x,
        pos_wrapped.y * pos_field.y - 1.0,
        pos_eye.z + (pos_wrapped.z - 0.5) * pos_field.z,
    );

    var pos_landing = swayed(point_at_height(pos_center, pos_flake_fall, rim_height), in.seed.x);
    if length(pos_landing.xz) >= rim_radius.y {
        pos_landing = swayed(point_at_height(pos_center, pos_flake_fall, 0.0), in.seed.x);
    }
    let t_2 = (pos_center.y - pos_landing.y) / pos_flake_fall.y;
    let in_pot_mouth = pos_landing.y > 0.0 && length(pos_landing.xz) < rim_radius.x;
    let gone = t_2 > t_1 || (t_2 > 0.0 && in_pot_mouth);
    if gone {
        return retval;
    }

    //let t_3 = cos(globals.time * 3.42);
    let t_3 = -t_2;
    let sample = u32((t_3 + snowfall.first_sample_age) / snowfall.sample_step);
    let strength = snowfall.strength[sample / 4u][sample % 4u];
    if in.seed.x > strength {
        return retval;
    }

    var pos_flake = swayed(pos_center, in.seed.x);
    if t_2 > 0.0 {
        pos_flake = pos_landing;
    }
    let melt = 1.0 - clamp(t_2 / t_1, 0.0, 1.0);

    let right_normal = view.world_from_view[0].xyz;
    let up_normal = view.world_from_view[1].xyz;
    let eye_distance = length(pos_eye - pos_flake);
    let size = max(mix(flake_size.x, flake_size.y, in.seed.y), eye_distance * 0.0009) * melt;
    let pos_flake_corner = pos_flake + (right_normal * in.corner.x + up_normal * in.corner.y) * size;
    retval.position = position_world_to_clip(pos_flake_corner);

    let distance_fade = smoothstep(nearest, nearest + 1.0, eye_distance)
        * (1.0 - smoothstep(far_fade.x, far_fade.y, eye_distance));
    retval.fade = distance_fade * melt;
    return retval;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let col_snow_tint = vec3<f32>(0.5, 0.54, 0.6);

    let flake = 1.0 - smoothstep(0.35, 1.0, length(in.corner));
    let col_tint = col_snow_tint + vec3<f32>(0.3, 0.33, 0.42) * snowfall.flash;
    var retval = vec4<f32>(col_tint * flake * in.fade, 0.0);
    return retval;
}
