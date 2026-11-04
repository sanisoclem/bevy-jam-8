import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::{view, globals};
import super::noise::{hash3, fractal_noise};

struct SkyMotion {
    since_strike: f32,
    strike_number: u32,
    flash: f32,
    intensity: f32,
    activity: f32,
    star_shine: f32,
    star_tick: f32,
}

struct Sky {
    haze: vec4<f32>,
    star_lines: f32,
    @align(16) motion: SkyMotion,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: Sky;

fn stars(sky_normal: vec3<f32>, block: f32, cells: f32, density: f32) -> vec3<f32> {
    let pos_cell = floor(sky_normal * cells);
    let pos_random = hash3(pos_cell);
    if pos_random.z > density {
        return vec3<f32>(0.0);
    }

    let col_extra = hash3(pos_cell + 91.0);
    let star_normal = normalize(pos_cell + 0.04 + pos_random * vec3<f32>(0.92));
    let brightness = pow(col_extra.x, 4.0);
    let radius = block * mix(0.5, 0.8, brightness);
    let lit = step(length(sky_normal - star_normal), radius);
    let twinkle = 0.8 + 0.2 * sin(globals.time * mix(0.6, 2.2, col_extra.y) + col_extra.z * 90.0);
    let col_tint = mix(vec3<f32>(1.0, 0.72, 0.5), vec3<f32>(0.62, 0.78, 1.0), col_extra.y);
    let chosen = fract(col_extra.z * 17.0 + sky.motion.star_tick * 0.618) < 0.18;
    let strength = sky.motion.star_shine * mix(0.15, 1.0, sky.motion.intensity);
    let shine = select(0.0, strength * 1.8 * sky.motion.activity, chosen);
    return col_tint * lit * (twinkle * mix(0.05, 1.1, brightness) + shine);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let intensity_drop = 0.6;
    let col_cloud_shadow = vec3<f32>(0.012, 0.014, 0.02);
    let col_cloud_light = vec3<f32>(0.05, 0.056, 0.074);
    let col_flash_glow = vec3<f32>(0.62, 0.68, 0.95);

    let ray_normal = normalize(in.world_position.xyz - view.world_position);
    let drop = sky.motion.activity * intensity_drop * sky.motion.intensity;
    var retval: vec4<f32>;
    let ceiling_scale = exp(-drop * 0.35);
    let ceiling = ray_normal.xz / (max(ray_normal.y, 0.0) + 0.22) * ceiling_scale;
    let t_1 = vec2<f32>(0.035, 0.012) * globals.time;
    let billow = fractal_noise(vec3<f32>(ceiling * 1.4 + t_1, globals.time * 0.02));
    let detail = fractal_noise(vec3<f32>(ceiling * 3.7 + t_1 * 2.0, 5.0));
    let cover = smoothstep(0.32, 0.68, billow * 0.7 + detail * 0.3);

    let col_cloud = mix(col_cloud_shadow, col_cloud_light, cover);
    let col_lit_cloud = col_cloud + col_flash_glow * sky.motion.flash * (0.22 + cover * cover);
    let col_storm = mix(col_cloud_shadow, col_lit_cloud, smoothstep(0.0, 0.16, ray_normal.y));
    //retval = vec4<f32>(col_storm, t_1);
    //retval = vec4<f32>(col_storm * t_1, 1.0);
    retval = vec4<f32>(col_storm, 1.0);
    return retval;
}
