import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import super::screen::{screen_size, screen_texture};

struct Lens {
    blur: f32,
    shake: vec2<f32>,
}

@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> lens: Lens;

fn tap(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(screen_texture, screen_sampler, uv, 0.0).rgb;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = screen_size();
    let aspect = vec2<f32>(size.y / size.x, 1.0);
    let uv = in.uv + lens.shake * aspect;
    var retval: vec4<f32>;
    if lens.blur < 0.00002 {
        retval = vec4<f32>(tap(uv), 1.0);
    } else {
        let taps = 20u;
        var col_sum = vec3<f32>(0.0);
        for (var index = 0u; index < taps; index += 1u) {
            let angle = f32(index) * 2.39996;
            let radius = sqrt((f32(index) + 0.5) / f32(taps));
            col_sum += tap(uv + vec2<f32>(cos(angle), sin(angle)) * radius * lens.blur * aspect);
        }
        retval = vec4<f32>(col_sum / f32(taps), 1.0);
    }
    return retval;
}
