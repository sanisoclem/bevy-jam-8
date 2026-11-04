import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};
import bevy_pbr::render::mesh_view_bindings::view;

const TAU = 6.283185;
const BANDS_PER_LEG = 48u;

struct ShadowWaves {
    levels: array<vec4<f32>, 36>,
    band_count: f32,
    strike_level: f32,
    limit_scale: f32,
    _webgl2_padding: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> waves: ShadowWaves;

fn band_level(leg: u32, band: f32) -> f32 {
    if band < 0.0 || band >= waves.band_count {
        return 0.0;
    }
    let slot = leg * BANDS_PER_LEG + u32(band);
    return waves.levels[slot / 4u][slot % 4u];
}

fn spread_level(leg: u32, band: f32, neighbour_share: f32) -> f32 {
    let neighbours = band_level(leg, band - 1.0) + band_level(leg, band + 1.0);
    return band_level(leg, band) * (1.0 - 2.0 * neighbour_share) + neighbours * neighbour_share;
}

@fragment
fn fragment(vertex_output: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let first_leg = vec2<f32>(0.866, 0.5);
    let second_leg = vec2<f32>(0.0, -1.0);
    let third_leg = vec2<f32>(-0.866, 0.5);
    let shadow_line = vec2<f32>(0.9, 10.0);
    let note_padding = 0.2;
    let line_fade = 0.05;
    let neighbour_share = 0.1;
    //const WARP_PER_LEVEL = 0.02;
    //const LARGEST_WARP = 0.12;
    let warp_per_level = 0.5;
    let largest_warp = 0.42;

    var in = vertex_output;
    let position = in.world_position.xz;
    let radius = length(position);
    let line = log(radius / shadow_line.x) / log(shadow_line.y / shadow_line.x);
    if line > 0.0 && line < 1.0 {
        let legs = mat3x2<f32>(first_leg, second_leg, third_leg);
        let toward = position / radius;
        let col_closeness = toward * legs;
        var leg = 2u;
        if col_closeness.x >= col_closeness.y && col_closeness.x >= col_closeness.z {
            leg = 0u;
        } else if col_closeness.y >= col_closeness.z {
            leg = 1u;
        }
        let reach = smoothstep(0.5, 0.95, col_closeness[leg]);

        let band_place = (line - note_padding) / (1.0 - 2.0 * note_padding) * waves.band_count;
        let place = band_place - 0.5;
        let band = floor(place);
        let blend = smoothstep(0.0, 1.0, place - band);
        let level = mix(
            spread_level(leg, band, neighbour_share),
            spread_level(leg, band + 1.0, neighbour_share),
            blend,
        );
        let wanted = warp_per_level * (level + waves.strike_level);
        let largest = largest_warp * waves.limit_scale;
        let fade = smoothstep(0.0, line_fade, line) * (1.0 - smoothstep(1.0 - line_fade, 1.0, line));
        let amplitude = largest * (1.0 - exp(-wanted / largest)) * fade * reach;

        // TODO: remopve this branch
        if amplitude > 0.0 {
            let camera = view.world_position.xz;
            let facing = (camera / max(length(camera), 0.001) * legs)[leg];
            let waves_per_band = 2.0 + facing;
            let from_middle = band_place - 0.5 * waves.band_count;
            let wave = sin(TAU * from_middle * waves_per_band);

            let across = vec2<f32>(-toward.y, toward.x);
            let mirror_side = clamp((across * legs)[leg] / 0.05, -1.0, 1.0);
            let ground = position + across * mirror_side * wave * amplitude;
            in.world_position.x = ground.x;
            in.world_position.z = ground.y;
        }
    }

    let pbr_input = pbr_input_from_standard_material(in, is_front);
    var retval: FragmentOutput;
    retval.color = main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
    return retval;
}
