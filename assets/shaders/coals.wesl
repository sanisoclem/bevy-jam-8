import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::globals;
import super::noise::{hash3, value_noise};

struct Blaze {
    flare: f32,
    _webgl2_padding: vec3<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> blaze: Blaze;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let col_coalz = vec3<f32>(0.006, 0.0055, 0.007);
    let col_smoulder = vec3<f32>(0.22, 0.008, 0.004);
    let col_ember = vec3<f32>(0.95, 0.075, 0.008);
    // TODO: how to make a deep green???
    let col_evil = vec3<f32>(0.0, 3.42, 0.0);
    let flare_heat = 0.05;

    let pos_world = in.world_position.xyz;
    let world_normal = normalize(in.world_normal);

    let pos_drift = vec3<f32>(0.0, globals.time * 0.05, 0.0);
    let patches = value_noise(pos_world * 2.6 + pos_drift);
    let veins = value_noise(pos_world * 9.0 + pos_drift * 1.5);
    let breath = sin(globals.time * 0.55 + patches * 7.0) * 0.1;
    let facet = hash3(floor(world_normal * 31.0)).x - 0.5;
    let cooling_edge = smoothstep(0.45, 1.0, length(pos_world.xz));
    let heat = patches * 0.6 + veins * 0.4 + breath + facet * 0.2 - cooling_edge * 0.22
        + blaze.flare * flare_heat;

    let col_dull = mix(vec3<f32>(0.0), col_smoulder, smoothstep(0.46, 0.54, heat));
    let col_lit = mix(col_dull, col_ember, smoothstep(0.56, 0.66, heat));
    let col_glow = mix(col_lit, col_evil, smoothstep(0.7, 0.84, heat));
    var retval = vec4<f32>(col_coalz * (0.3 + max(world_normal.y, 0.0)) + col_glow, 1.0);
    return retval;
}
