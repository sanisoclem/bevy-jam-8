import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::{view, globals};
import super::noise::value_noise;

struct Fluid {
    color: vec4<f32>,
    icon: f32,
    _webgl2_padding: vec3<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> fluid: Fluid;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var sheet: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var sheet_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let pos = in.world_position.xz;
    let radius = length(pos) / 0.7;
    let angle = atan2(pos.y, pos.x) + t * mix(0.3, 0.1, radius);
    let current = radius * 7.0 + angle / 6.283185;
    let swirl = vec2<f32>(cos(angle), sin(angle)) * radius;
    let pieces = smoothstep(0.5, 0.6, value_noise(vec3<f32>(swirl * 2.5, radius * 8.0)));
    var line = (1.0 - smoothstep(0.02, 0.05, abs(fract(current) - 0.5))) * pieces;

    var etch = 0.0;
    if fluid.icon >= 0.0 {
        // don't draw swirls near icon
        line *= smoothstep(0.45, 0.75, radius);
        let camera_flat = view.world_position.xz;
        let toward_camera = camera_flat / max(length(camera_flat), 0.001);
        let across = vec2<f32>(toward_camera.y, -toward_camera.x);
        let icon_uv = vec2<f32>(dot(pos, across), dot(pos, toward_camera)) / 0.9 + 0.5;
        if all(icon_uv >= vec2<f32>(0.0)) && all(icon_uv <= vec2<f32>(1.0)) {
            let sheet_size = vec2<f32>(textureDimensions(sheet));
            let corner = vec2<f32>(fluid.icon * 192.0, 2344.0);
            etch = clamp(textureSampleLevel(sheet, sheet_sampler, (corner + icon_uv * 192.0) / sheet_size, 0.0).a, 0.0, 1.0);
        }
    }

    let col_hue = fluid.color.rgb / max(max(fluid.color.r, fluid.color.g), max(fluid.color.b, 0.001));
    var retval = vec4<f32>(col_hue * (0.1 + 1.6 * line) * mix(1.0, 1000.0, etch), 1.0);
    //retval = vec4<f32>(vec3<f32>(etch)* fluid.color.rgb * 10000.0, 1.0);
    //retval = vec4<f32>(col_hue * (0.1 + 1.6 * line) * mix(100.0, 0.0, etch), 1.0);
    return retval;
}
