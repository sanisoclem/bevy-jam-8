const GAMMA = 2.2;

@group(0) @binding(0) var screen_texture: texture_2d<f32>;

fn screen_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(screen_texture));
}

fn scene_color(pixel: vec2<f32>) -> vec3<f32> {
    let last_texel = vec2<i32>(textureDimensions(screen_texture)) - 1;
    let texel = clamp(vec2<i32>(pixel), vec2<i32>(0), last_texel);
    return max(textureLoad(screen_texture, texel, 0).rgb, vec3<f32>(0.0));
}

fn scene_tone(pixel: vec2<f32>) -> vec3<f32> {
    return pow(scene_color(pixel), vec3<f32>(1.0 / GAMMA));
}

fn output_color(tone: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(pow(tone, vec3<f32>(GAMMA)), 1.0);
}
