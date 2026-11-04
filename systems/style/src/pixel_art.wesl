import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import super::screen::{output_color, scene_tone, screen_size};

struct PixelArt {
    pixel_lines: f32,
    tone_bands: f32,
    hue_levels: f32,
    outline: f32,
}

@group(0) @binding(2) var<uniform> pixel_art: PixelArt;

fn block_tone(block: vec2<f32>, block_width: f32) -> vec3<f32> {
    return scene_tone((block + 0.5) * block_width);
}

fn lightness(col_tone: vec3<f32>) -> f32 {
    return dot(col_tone, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let block_width = max(round(screen_size().y / pixel_art.pixel_lines), 1.0);
    let block = floor(in.position.xy / block_width);
    let col_tone = block_tone(block, block_width);
    let light = lightness(col_tone);

    let col_shadow_tint = mix(vec3<f32>(0.66, 0.86, 1.7), vec3<f32>(1.0), smoothstep(0.08, 0.42, light));
    let col_hue = round(col_tone / max(light, 0.0001) * col_shadow_tint * pixel_art.hue_levels) / pixel_art.hue_levels;
    let band = round(sqrt(light) * pixel_art.tone_bands) / pixel_art.tone_bands;

    var brighter = 0.0;
    for (var x = -1.0; x <= 1.0; x += 1.0) {
        for (var y = -1.0; y <= 1.0; y += 1.0) {
            let neighbour = lightness(block_tone(block + vec2<f32>(x, y), block_width));
            if neighbour > light * 1.5 + 0.1 {
                brighter += 1.0;
            }
        }
    }
    let ink = select(1.0, 1.0 - pixel_art.outline, brighter >= 3.0);

    var retval = output_color(max(col_hue * band * band * ink, vec3<f32>(0.0)));
    return retval;
}
