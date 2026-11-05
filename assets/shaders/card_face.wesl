import super::noise::value_noise;

// code shared by 3d and UI
struct CardFace {
    panel: vec4<f32>,
    icon: f32,
    outer_frame: f32,
    inner_frame: f32,
    _webgl2_padding: f32,
}

fn card_mask(sheet: texture_2d<f32>, sheet_sampler: sampler, face: CardFace, uv: vec2<f32>) -> f32 {
    // TODO: calculate card_size from asset?
    let card_size = vec2<f32>(422.0, 586.0);
    let corner = vec2<f32>(face.outer_frame * card_size.x, card_size.y);
    let opp = corner + clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)) * card_size;

    // only use alpha channel
    // TODO: use other channels for adding a highlight shimmer effect
    return textureSampleLevel(sheet, sheet_sampler, opp / vec2<f32>(textureDimensions(sheet)), 0.0).a;
}

fn roughify(uv: vec2<f32>) -> f32 {
    // TODO: doesn't look like paper
    let xy = uv * vec2<f32>(422.0, 586.0);
    return value_noise(vec3<f32>(xy * 0.1, 0.0)) * 0.6
        + value_noise(vec3<f32>(xy.x * 0.02, xy.y * 0.25, 3.0)) * 0.4;
}

fn border(grain: f32) -> vec3<f32> {
    //return vec3<f32>(0.0, 0.0, 0.72) * grain * 1.;
    return vec3<f32>(0.85, 0.82, 0.72) * (1.0 - 0.08 * grain);
}

fn card_color(sheet: texture_2d<f32>, sheet_sampler: sampler, face: CardFace, uv: vec2<f32>) -> vec3<f32> {
    // TODO: calculate card_size from asset?
    let card_size = vec2<f32>(422.0, 586.0);
    let sheet_size = vec2<f32>(textureDimensions(sheet));
    let outer = vec2<f32>(face.outer_frame * card_size.x, 0.0);

    // 3rd row in the spritesheet
    let inner = vec2<f32>(face.inner_frame * card_size.x, 2.0 * card_size.y);
    let panel = textureSampleLevel(sheet, sheet_sampler, (inner + vec2<f32>(0.0, card_size.y) + uv * card_size) / sheet_size, 0.0).a;
    var retval = mix(vec3<f32>(0.85, 0.82, 0.72), face.panel.rgb, panel);

    var col_mark = vec3<f32>(0.95, 0.94, 0.9);
    if dot(face.panel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)) > 0.45 {
        col_mark = vec3<f32>(0.02, 0.02, 0.025);
    }

    // TODO: calculate icon size or load from uniform?
    let icon_cell = 192.0;
    let icon_height = 0.36;
    let icon_uv = (uv - 0.5) * vec2<f32>(card_size.x / card_size.y, 1.0) / icon_height + 0.5;
    if all(icon_uv >= vec2<f32>(0.0)) && all(icon_uv <= vec2<f32>(1.0)) {
        // 5th row
        let corner = vec2<f32>(face.icon * icon_cell, 4.0 * card_size.y);
        let icon = textureSampleLevel(sheet, sheet_sampler, (corner + icon_uv * icon_cell) / sheet_size, 0.0).a;
        retval = mix(retval, col_mark, icon * panel);
    }

    let col_outlines = vec3<f32>(0.03, 0.03, 0.035);
    let outer_outline = textureSampleLevel(sheet, sheet_sampler, (outer + uv * card_size) / sheet_size, 0.0).a;
    let inner_outline = textureSampleLevel(sheet, sheet_sampler, (inner + uv * card_size) / sheet_size, 0.0).a;
    retval = mix(retval, col_outlines, max(outer_outline, inner_outline));
    return retval;
}
