fn pcg3d(seed: vec3<u32>) -> vec3<u32> {
    var v = seed * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v ^= v >> vec3<u32>(16u);
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return v;
}

fn hash3(cell: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(pcg3d(vec3<u32>(vec3<i32>(cell)))) / 4294967295.0;
}

fn value_noise(position: vec3<f32>) -> f32 {
    let cell = floor(position);
    let blend = smoothstep(vec3<f32>(0.0), vec3<f32>(1.0), fract(position));
    let corner = vec2<f32>(0.0, 1.0);
    return mix(
        mix(
            mix(hash3(cell + corner.xxx).x, hash3(cell + corner.yxx).x, blend.x),
            mix(hash3(cell + corner.xyx).x, hash3(cell + corner.yyx).x, blend.x),
            blend.y,
        ),
        mix(
            mix(hash3(cell + corner.xxy).x, hash3(cell + corner.yxy).x, blend.x),
            mix(hash3(cell + corner.xyy).x, hash3(cell + corner.yyy).x, blend.x),
            blend.y,
        ),
        blend.z,
    );
}
