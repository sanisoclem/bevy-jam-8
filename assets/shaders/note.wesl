import bevy_pbr::render::forward_io::{FragmentOutput, VertexOutput};
import bevy_pbr::render::mesh_view_bindings::globals;
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};
import super::noise::value_noise;

struct Note {
    // packed u8 icon index x3, age, 
    // flags: active, erased, focused
    lines: array<vec4<u32>, 16>,
    length: f32,
    // icon index of the hovered card
    current: u32,
    _webgl2_padding: vec2<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> note: Note;
@group(constants::MATERIAL_BIND_GROUP) @binding(101)
var sheet: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102)
var sheet_sampler: sampler;

@fragment
fn fragment(vertex_output: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let xy = vertex_output.uv * vec2<f32>(4.8, 17.0);

    //dist from nearest edge (dynamic height)
    let edge = min(min(xy.x, 4.8 - xy.x), min(xy.y, note.length - xy.y));
    if edge < -0.06 {
        discard;
    }

    // noise edge 
    // let alpha = edge + value_noise(vec3<f32>(xy * 6.0, 0.0)) * 0.1;
    var alpha = smoothstep(
            -0.02,
            0.02,
            edge + 0.08 * (value_noise(vec3<f32>(xy * 3.0, 0.0)) - 0.5),
        );
    // alpha = 1.0;

    var col_note = vec3<f32>(0.5, 0.82, 0.72)
        * (1.0 - 0.08 * value_noise(vec3<f32>(xy * vec2<f32>(8.0, 30.0), 1.0)));
    var col_glow = vec3<f32>(0.0);
    // let temp = col_note;

    let line = i32(floor(xy.y - 0.5));
    let cell = i32(floor((xy.x - 0.3) / 0.84));
    if line >= 16 || line < 0 || cell < 0 || cell >= 5 {
        discard;
    }

    let entry = note.lines[line];

    // active
    if (entry.z & 1u) != 0u {
        let focused = (entry.z & 4u) != 0u;
        let erased = (entry.z & 2u) != 0u;
        let seed = f32(line * 5 + cell);
        let uv = vec2<f32>(xy.x - 0.3 - (f32(cell) + 0.5) * 0.84, xy.y - f32(line) - 1.0) / 0.8
            + 0.5;

        // stagger age per cell
        let age_secs = f32(entry.y) / 1000.0;
        let t_2 = select(age_secs - 0.4 - f32(cell) * 0.2, 5.0, erased);
        // col_note = col_note * t_2;

        // TODO: just draw + and = icons
        let age_scaled = 1.0 - saturate(t_2 / 1.5);
        let mid = uv - 0.5;
        let mid_clamped = vec2<f32>(mid.x - clamp(mid.x, -0.22, 0.22), mid.y);
        let radius = 0.1 * age_scaled * age_scaled;

        var density = 0.0;
        var current = false;
        if cell == 1 {
            let plus = min(
                    length(mid_clamped),
                    length(vec2<f32>(mid.x, mid.y - clamp(mid.y, -0.22, 0.22))),
                );
            density = saturate(0.5 - (plus - 0.02) / (2.0 * radius + 0.025));
        } else if cell == 3 {
            let mid_clamped = vec2<f32>(mid.x - clamp(mid.x, -0.22, 0.22), mid.y);
            var stroke = length(vec2<f32>(mid_clamped.x, abs(mid_clamped.y) - 0.09));
            density = saturate(0.5 - (stroke - 0.02) / (2.0 * radius + 0.025));
        } else {
            let sheet_size = vec2<f32>(textureDimensions(sheet));
            let icon = (entry.x >> (8u * u32(cell / 2))) & 0xffu;
            current = icon == note.current;
            let corner = vec2<f32>(f32(icon) * 192.0, 4.0 * 586.0);
            density += textureSampleLevel(
                    sheet,
                    sheet_sampler,
                    (corner + saturate(uv) * 192.0) / sheet_size,
                    0.0,
                ).a;
        }

        let col_ink = vec3<f32>(0.01, 0.01, 0.04);
        let coverage = smoothstep(0.0, 0.3, t_2) * density
            * select(1.0, 1.0 - smoothstep(0.0, 0.3, age_secs), erased);
        col_note = mix(col_note, col_ink, coverage);

        if focused {
            col_glow = select(vec3<f32>(6.0, 3.0, 0.75), vec3<f32>(0.75, 3.0, 6.0), current) * coverage
                * (0.85 + 0.15 * sin(globals.time * 5.0));
        }
    }

    var pbr_input = pbr_input_from_standard_material(vertex_output, is_front);
    pbr_input.material.base_color = vec4<f32>(col_note * 0.6, alpha);
    pbr_input.material.emissive = vec4<f32>(col_note * 0.5 + col_glow, 0.0);
    var retval: FragmentOutput;
    retval.color = main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
    // retval.color = vec4<f32>(col_note, alpha);
    return retval;
}
