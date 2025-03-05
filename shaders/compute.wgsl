@group(0) @binding(0) var<storage, read_write> positions: array<vec4f>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4f>;
@group(0) @binding(2) var<storage, read_write> masses: array<f32>;

const G: f32 = 6e-8;
const D: f32 = 0.0005;

@compute @workgroup_size(1) fn cs_entry(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    if id.x == id.y {
      return;
    }

    // Calc new velocity
    let m1 = masses[id.x];
    let m2 = masses[id.y];
    let r = positions[id.y] - positions[id.x];
    // operation is unoptomized...
    let l = inverseSqrt(r.x * r.x + r.y * r.y + r.z * r.z);
    let f = ( m1 * m2 * G / (l * l * l) ) * r;
    let a = f * D / m1;

    velocities[id.x] += a;

    // Move position, not sure if this will cause issues...
    positions[id.x] += velocities[id.x];
    
}
