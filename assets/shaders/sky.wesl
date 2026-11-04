
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
    storm: f32,
    bolt_heading: f32,
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

fn zigzag(position: f32, seed: f32) -> f32 {
    let cell = floor(position);
    let here = hash3(vec3<f32>(cell, seed, 1.0)).x;
    let next = hash3(vec3<f32>(cell + 1.0, seed, 1.0)).x;
    return mix(here, next, position - cell) - 0.5;
}

fn bolt_wander(height: f32, seed: f32) -> f32 {
    var retval = zigzag(height * 7.0, seed) * 0.3
        + zigzag(height * 23.0, seed + 5.0) * 0.1
        + zigzag(height * 71.0, seed + 9.0) * 0.035;
    return retval;
}

fn bolt_line(gap: f32, bolt_width: f32) -> f32 {
    let core = 1.0 - smoothstep(0.0, bolt_width, abs(gap));
    let halo = exp(-abs(gap) / 0.035) * 0.16;
    return max(core, halo);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let intensity_drop = 0.6;
    let col_cloud_shadow = vec3<f32>(0.012, 0.014, 0.02);
    let col_cloud_light = vec3<f32>(0.05, 0.056, 0.074);
    let col_flash_glow = vec3<f32>(0.62, 0.68, 0.95);
    let col_bolt_glow = vec3<f32>(3.4, 3.6, 4.6);
    let bolt_top = 0.46;
    let bolt_width = 0.0026;

    let ray_normal = normalize(in.world_position.xyz - view.world_position);
    let drop = sky.motion.activity * intensity_drop * sky.motion.intensity;
    var retval: vec4<f32>;
    if sky.storm > 0.5 {
        let ceiling_scale = exp(-drop * 0.35);
        let ceiling = ray_normal.xz / (max(ray_normal.y, 0.0) + 0.22) * ceiling_scale;
        let t_1 = vec2<f32>(0.035, 0.012) * globals.time;
        let billow = fractal_noise(vec3<f32>(ceiling * 1.4 + t_1, globals.time * 0.02));
        let detail = fractal_noise(vec3<f32>(ceiling * 3.7 + t_1 * 2.0, 5.0));
        let cover = smoothstep(0.32, 0.68, billow * 0.7 + detail * 0.3);

        let col_cloud = mix(col_cloud_shadow, col_cloud_light, cover);
        let col_lit_cloud = col_cloud + col_flash_glow * sky.motion.flash * (0.22 + cover * cover);
        let col_storm = mix(col_cloud_shadow, col_lit_cloud, smoothstep(0.0, 0.16, ray_normal.y));
        // retval = vec4<f32>(col_storm, 1.0);
        if sky.motion.flash < 0.01 {
            retval = vec4<f32>(col_storm, 1.0);
        } else {
            let height = ray_normal.y;
            let strike = f32(sky.motion.strike_number);
            let angle_from_bolt = atan2(ray_normal.x, -ray_normal.z) - sky.bolt_heading;
            let across = atan2(sin(angle_from_bolt), cos(angle_from_bolt));
            let reach = 1.0 - smoothstep(bolt_top * 0.8, bolt_top, height);
            let trunk = across - bolt_wander(height, strike);

            let col_dice = hash3(vec3<f32>(strike, 3.0, 7.0));
            let fork_height = mix(0.1, 0.3, col_dice.y);
            let fork_side = select(-1.0, 1.0, col_dice.z > 0.5);
            let fork_run = max(fork_height - height, 0.0);
            let fork_gap = trunk - fork_side * fork_run * 0.55;
            let fork_line = bolt_line(fork_gap - bolt_wander(height, strike + 11.0) * fork_run * 4.0, bolt_width);
            let fork = select(0.0, fork_line, height < fork_height);

            let bolt = max(bolt_line(trunk, bolt_width), fork) * reach;
            retval = vec4<f32>(col_storm + col_bolt_glow * sky.motion.flash * bolt, 1.0);
        }
    } else {
        let squeeze = exp(-drop);
        let squeezed_normal = normalize(vec3<f32>(ray_normal.x * squeeze, ray_normal.y, ray_normal.z * squeeze));
        let t_2 = globals.time * 0.0035;
        let t_3 = cos(0.01 * globals.time);
        let cosine = cos(t_2);
        let sine = sin(t_2);
        let sky_normal = vec3<f32>(
            squeezed_normal.x * cosine + squeezed_normal.z * sine,
            squeezed_normal.y,
            squeezed_normal.z * cosine - squeezed_normal.x * sine,
        );

        let block_width = max(round(view.viewport.w / sky.star_lines), 1.0);
        let block = length(fwidth(ray_normal)) * block_width * (1.0 + max(drop, 0.0) * 0.6);
        let crowd = 0.04 * sky.motion.intensity * sky.motion.activity;
        let col_field = stars(sky_normal, block, 58.0, 0.012 + crowd)
            + stars(sky_normal, block, 113.0, 0.012 + crowd);

        let col_night = mix(sky.haze.rgb, vec3<f32>(0.0015, 0.002, 0.006), smoothstep(0.0, 0.34, ray_normal.y));
        let hugging = exp(-max(ray_normal.y, 0.0) / 0.07);
        let strike_fade = sky.motion.activity * exp(-sky.motion.since_strike / 0.35);
        let col_horizon_pulse = vec3<f32>(0.03, 0.02, 0.06) * strike_fade * hugging;
        retval = vec4<f32>(col_night + col_horizon_pulse + col_field, 1.0);
    }
    //retval = vec4<f32>(col_storm, 1.0);
    //retval = vec4<f32>(col_storm, t_1);
    //retval = vec4<f32>(col_storm * t_1, 1.0);
    return retval;
}
