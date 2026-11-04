import bevy_pbr::render::mesh_view_bindings::{view, globals};
import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import super::noise::{value_noise, fractal_noise};

struct Glow {
    strength: f32,
    _webgl2_padding: vec3<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> glow: Glow;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) origin: vec3<f32>,
    @location(2) corner: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) corner: vec2<f32>,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let glow_size = 0.9;

    var retval: VertexOutput;
    retval.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    retval.corner = in.corner;
    if glow.strength < 0.002 {
        return retval;
    }

    let world_from_local = get_world_from_local(in.instance_index);
    let pos_center = (world_from_local * vec4<f32>(in.origin, 1.0)).xyz;
    let right_normal = view.world_from_view[0].xyz;
    let up_normal = view.world_from_view[1].xyz;
    let pos_offset = (right_normal * in.corner.x + up_normal * in.corner.y) * glow_size;
    retval.position = position_world_to_clip(pos_center + pos_offset);
    return retval;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let col_glow = vec3<f32>(0.2, 1.0, 0.3);
    let glow_peak = 0.22;
    let rise_speed = 0.2;
    let wobble = 0.25;
    let tilt = mat3x3<f32>(
        vec3<f32>(0.0, 0.8, 0.6),
        vec3<f32>(-0.8, 0.36, -0.48),
        vec3<f32>(-0.6, -0.48, 0.64),
    );

    let t_1 = globals.time * 0.08;
    let broad = value_noise(vec3<f32>(in.corner * 1.5, t_1));
    let fine = value_noise(vec3<f32>(in.corner * 3.7, t_1 * 1.7 + 9.0));
    let outline_wobble = mix(broad, fine, 0.35) * 2.0 - 1.0;
    let wobbled_distance = length(in.corner) * (1.0 + wobble * outline_wobble);

    let squared = wobbled_distance * wobbled_distance;
    let brightness = mix(exp(-squared / 0.35), exp(-squared / 0.08), 0.4);
    let falloff = brightness * (1.0 - smoothstep(0.35, 0.95 * (1.0 - wobble), wobbled_distance));

    let rising = in.corner - vec2<f32>(0.0, globals.time * rise_speed);
    let pos_churning = vec3<f32>(rising * 3.0, globals.time * 0.25);
    let wisps = smoothstep(0.3, 0.7, fractal_noise(tilt * pos_churning));

    let wander = value_noise(vec3<f32>(globals.time * 0.3, 0.5, 0.5)) * 2.0 - 1.0;
    let swell = 1.0 + 0.25 * wander;

    let shape = falloff * mix(1.0, wisps, 0.7);
    let col_light = col_glow * glow_peak * glow.strength * swell * shape;
    var retval = vec4<f32>(col_light, 0.0);
    return retval;
}
