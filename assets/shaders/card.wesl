import bevy_pbr::render::forward_io::{FragmentOutput, VertexOutput};
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};
import bevy_pbr::render::mesh_functions::get_world_from_local;
import super::card_face::{CardFace, border, card_color, card_mask, roughify};
import super::noise::value_noise;

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> card: CardFace;
@group(constants::MATERIAL_BIND_GROUP) @binding(101)
var sheet: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102)
var sheet_sampler: sampler;

@fragment
fn fragment(vertex_output: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let uv = vertex_output.uv;

    var pbr_input = pbr_input_from_standard_material(vertex_output, is_front);
    //let pos_dx = dpdx(vertex_output.world_position.xyz);
    //let pos_dy = dpdy(vertex_output.world_position.xyz);
    //let uv_dx = dpdx(uv);
    //let uv_dy = dpdy(uv);
    //let det = uv_dx.x * uv_dy.y - uv_dx.y * uv_dy.x;
    //let inverse_det = 1.0 / select(det, 1e-12, abs(det) < 1e-12);
    //let pos_across = (pos_dx * uv_dy.y - pos_dy * uv_dx.y) * inverse_det;
    //let pos_down = (pos_dy * uv_dx.x - pos_dx * uv_dy.x) * inverse_det;
    //let grain = roughify(uv);
    //let grain_dx = dpdx(grain);
    //let grain_dy = dpdy(grain);

    //let grain = roughify(uv);
    let world_from_local = get_world_from_local(vertex_output.instance_index);
    let pos_across = world_from_local[0].xyz * (422.0 / 586.0);
    let pos_down = -world_from_local[1].xyz;
    let col_offwhite = vec3<f32>(0.85, 0.82, 0.72);

    let paper = card_mask(sheet, sheet_sampler, card, uv);
    let thickness = 0.02 * length(pos_down);
    let pos_sink = -pbr_input.V * thickness / max(dot(pbr_input.V, pbr_input.world_normal), 0.2);
    let behind = uv + vec2<f32>(
            dot(pos_sink, pos_across) / max(dot(pos_across, pos_across), 1e-12),
            dot(pos_sink, pos_down) / max(dot(pos_down, pos_down), 1e-12),
        );
    let alpha = max(paper, card_mask(sheet, sheet_sampler, card, behind));
    if alpha < 0.004 {
        discard;
    }

    var col_card = col_offwhite * 0.8;
    if is_front {
        col_card = card_color(sheet, sheet_sampler, card, uv);
    } else {
        // TODO: what to show behind the card
    }
    // wow
    col_card = mix(col_offwhite * 0.55, col_card, paper);

    // ugly freeze effect
    let edge = min(min(uv.x, 1.0 - uv.x) * (422.0 / 586.0), min(uv.y, 1.0 - uv.y));
    let rime = smoothstep(
            0.14,
            0.08,
            edge + value_noise(vec3<f32>(uv * vec2<f32>(20.0, 28.0), 0.0)) * 0.04,
        );
    col_card = mix(col_card, vec3<f32>(0.8, 0.9, 1.0), card.frost * max(rime, 0.2));

    pbr_input.material.base_color = vec4<f32>(col_card * 0.6, alpha);
    pbr_input.material.emissive = vec4<f32>(col_card * 0.5, 0.0);
    let lit = apply_pbr_lighting(pbr_input);
    var retval: FragmentOutput;
    retval.color = main_pass_post_lighting_processing(pbr_input, lit);
    return retval;
}
