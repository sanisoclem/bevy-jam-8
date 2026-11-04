import bevy_pbr::render::mesh_view_bindings::{view, globals};
import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;

struct CauldronInfo {
    center_y: f32,
    half_height: f32,
    radius: f32,
    ember_ceiling: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> cauld: CauldronInfo ;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) origin: vec3<f32>,
    @location(2) corner: vec2<f32>,
    @location(3) seed: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) age: f32,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let rise = 0.78;
    let ember_size = 0.014;

    let pace = mix(0.7, 1.3, in.seed.y);
    let t_1 = fract(globals.time * pace / 9.0 + in.seed.x);

    let height = in.origin.y + rise * mix(0.5, 1.0, in.seed.y) * t_1;
    let t_2 = in.seed.x * 40.0 + globals.time * 0.5;
    let wander = vec2<f32>(sin(t_2 + height * 3.0), cos(t_2 * 0.8 + height * 2.0));
    let lateral = in.origin.xz + wander * 0.09 * t_1;

    let reach = length(vec2<f32>(ember_size));
    let capped_height = min(height, cauld.ember_ceiling - reach);
    let across = (capped_height - cauld.center_y) / (cauld.half_height + reach);
    let hull_radius = (cauld.radius + reach) * sqrt(max(1.0 - across * across, 0.0));
    let radius = length(lateral);
    let pushed = lateral / radius * max(radius, hull_radius);
    let pos_center = vec3<f32>(pushed.x, capped_height, pushed.y);

    let world_from_local = get_world_from_local(in.instance_index);
    let pos_world_center = (world_from_local * vec4<f32>(pos_center, 1.0)).xyz;
    let right_normal = normalize(view.world_from_view[0].xyz * vec3<f32>(1.0, 0.0, 1.0));
    let pos_offset = right_normal * in.corner.x + vec3<f32>(0.0, in.corner.y, 0.0);

    var retval: VertexOutput;
    retval.position = position_world_to_clip(pos_world_center + pos_offset * ember_size);
    retval.age = t_1;

    // TODO: make different shapes? sparks, different velocitiess?
    return retval;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let col_hot = vec3<f32>(3.2, 0.8, 0.08);

    let fade = smoothstep(0.0, 0.08, in.age) * smoothstep(1.0, 0.6, in.age);
    var retval = vec4<f32>(mix(col_hot, vec3<f32>(0.9, 0.07, 0.01), in.age) * fade, 0.0);
    return retval;
}
