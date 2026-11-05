import bevy_ui_render::ui_vertex_output::UiVertexOutput;
import super::card_face::{CardFace, card_color, card_mask};

@group(1) @binding(0) var<uniform> card: CardFace;
@group(1) @binding(1) var sheet: texture_2d<f32>;
@group(1) @binding(2) var sheet_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let paper = card_mask(sheet, sheet_sampler, card, in.uv);
    let col_card = card_color(sheet, sheet_sampler, card, in.uv);
    var retval = vec4<f32>(col_card, paper);
    // retval = retval * 0.1;
    return retval;
}
