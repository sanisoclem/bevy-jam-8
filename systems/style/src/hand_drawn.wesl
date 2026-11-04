import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import super::noise::{hash3, value_noise};
import super::screen::{output_color, scene_color, scene_tone, screen_size};

struct HandDrawn {
    hatch_lines: f32,
    focal_length: f32,
    unwrapped_heading: f32,
    pitch: f32,
}

@group(0) @binding(2) var<uniform> drawing: HandDrawn;

fn lifted_light(pixel: vec2<f32>) -> f32 {
    let col_tone = scene_tone(pixel);
    return pow(max(max(col_tone.r, col_tone.g), col_tone.b), 0.62);
}

fn light_slope(pixel: vec2<f32>, reach: f32) -> vec2<f32> {
    let across = vec2<f32>(reach, 0.0);
    let down = vec2<f32>(0.0, reach);
    return vec2<f32>(
        lifted_light(pixel + across) - lifted_light(pixel - across),
        lifted_light(pixel + down) - lifted_light(pixel - down),
    );
}

fn light_tint(pixel: vec2<f32>) -> f32 {
    let tint_floor = 0.002;
    let col_scene = scene_color(pixel);
    return log((col_scene.r + tint_floor) / (col_scene.b + tint_floor));
}

fn stroke(board: vec2<f32>, angle: f32, coverage: f32) -> f32 {
    if coverage <= 0.0 {
        return 0.0;
    }

    let along = vec2<f32>(cos(angle), sin(angle));
    let bend = (value_noise(vec3<f32>(board * 0.12, angle * 5.0)) - 0.5) * 0.8;
    let row_position = dot(board, vec2<f32>(-along.y, along.x)) + bend;
    let row = floor(row_position);
    let stroke_length = 9.0;
    let run = dot(board, along) / stroke_length + hash3(vec3<f32>(row, angle, 1.0)).x * 5.0;
    let col_seed = hash3(vec3<f32>(row, floor(run), angle));
    if col_seed.x > coverage {
        return 0.0;
    }

    let pixel_size = drawing.hatch_lines / screen_size().y;
    let taper = smoothstep(0.0, 0.1, fract(run)) * smoothstep(1.0, 0.8, fract(run));
    let width = max(mix(0.07, 0.16, col_seed.y), 0.55 * pixel_size) * taper;
    let softness = 0.8 * pixel_size;
    return 1.0 - smoothstep(width - softness, width + softness, abs(fract(row_position) - 0.5));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pixel = in.position.xy;
    let size = screen_size();
    let light = lifted_light(pixel);

    let form_slope = light_slope(pixel, size.y * 0.012);
    var hatch_angle = 0.05;
    if length(form_slope) >= 0.04 {
        let form_angle_step = 0.3927;
        hatch_angle = round(atan2(form_slope.x, -form_slope.y) / form_angle_step) * form_angle_step;
    }

    let offset = (pixel - size * 0.5) / size.y;
    let ray_height = drawing.focal_length * sin(drawing.pitch) - offset.y * cos(drawing.pitch);
    var board: vec2<f32>;
    var col_coverage: vec3<f32>;
    if ray_height > 0.0 {
        let ahead = drawing.focal_length * cos(drawing.pitch) + offset.y * sin(drawing.pitch);
        let azimuth = atan2(offset.x, ahead) + drawing.unwrapped_heading;
        let elevation = atan2(ray_height, length(vec2<f32>(offset.x, ahead)));
        board = vec2<f32>(azimuth, -elevation) * drawing.focal_length * drawing.hatch_lines;
        col_coverage = smoothstep(vec3<f32>(0.3, 0.46, 0.64), vec3<f32>(0.55, 0.72, 0.86), vec3<f32>(light));
    } else {
        board = pixel / size.y * drawing.hatch_lines;
        hatch_angle += 1.5708;
        let col_lit_bands = vec3<f32>(0.36, 0.52, 0.7);
        col_coverage = smoothstep(col_lit_bands, col_lit_bands + 0.03, vec3<f32>(light));
    }

    let cross_hatch_turn = 1.25;
    let hatching = max(
        stroke(board, hatch_angle, col_coverage.x),
        max(
            stroke(board * 1.07, hatch_angle + cross_hatch_turn, col_coverage.y),
            stroke(board * 1.19, hatch_angle - cross_hatch_turn, col_coverage.z),
        ),
    );

    let outline_reach = size.y * 0.0009;
    let shake = vec2<f32>(
        value_noise(vec3<f32>(board * 0.1, 0.0)),
        value_noise(vec3<f32>(board * 0.1 + 9.0, 0.0)),
    ) - 0.5;
    let light_outline = smoothstep(0.03, 0.075, length(light_slope(pixel + shake * size.y * 0.0028, outline_reach)));
    var tint_outline = 0.0;
    if ray_height <= 0.0 {
        let across = vec2<f32>(outline_reach, 0.0);
        let down = vec2<f32>(0.0, outline_reach);
        let tint_slope = vec2<f32>(
            light_tint(pixel + across) - light_tint(pixel - across),
            light_tint(pixel + down) - light_tint(pixel - down),
        );
        tint_outline = smoothstep(0.14, 0.32, length(tint_slope));
    }
    let outlines = max(light_outline, tint_outline) * 0.85;

    let across = vec2<f32>(size.y * 0.006, 0.0);
    let down = vec2<f32>(0.0, across.x);
    let around = max(
        max(lifted_light(pixel + across), lifted_light(pixel - across)),
        max(lifted_light(pixel + down), lifted_light(pixel - down)),
    );
    let speck = smoothstep(0.04, 0.09, light - around);
    let marks = max(max(smoothstep(0.82, 0.95, light), speck), max(hatching, outlines));

    let col_tone = scene_tone(pixel);
    let brightest = max(max(col_tone.r, col_tone.g), max(col_tone.b, 0.001));
    let saturation = (brightest - min(min(col_tone.r, col_tone.g), col_tone.b)) / brightest;
    let col_chalk = vec3<f32>(0.93, 0.9, 0.82);
    let col_mark = mix(col_chalk, col_tone / brightest, saturation) * mix(0.5, 1.0, light);

    let col_board = vec3<f32>(0.055, 0.052, 0.08);
    var retval = output_color(mix(col_board, col_mark, marks));
    return retval;
}
