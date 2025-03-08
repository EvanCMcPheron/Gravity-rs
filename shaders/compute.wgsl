@group(0) @binding(0) var<storage, read_write> positions: array<vec4f>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4f>;
@group(0) @binding(2) var<storage, read_write> masses: array<f32>;

const G: f32 = 6e-3;
const D: f32 = 0.005;

@compute @workgroup_size(1) fn cs_entry(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    var x = id.x;
    var y = id.y;

    if x == y {
//      positions[x] += velocities[x] * D;
//      positions[x].w = 1;

      return;
    }

//    if y == 0 {
//      // This thread is wasted anyways (can't divide by zero), and incrimenting
//      // Position here garentees that it will be done once per body!
//      positions[x] += velocities[x] * D;
//      positions[x].w = 1;
//      return;
//    }

//    if y <= x {
//      // Between 1 and y=x, y is set back one to get every single value in [0,x]
//      // Then, once y>x, y = y so the y = x edgecase (an object's gravitation 
//      // on itself)
//      y -= 1;
//    }

    // Calc new velocity
    let m1 = masses[x];
    let m2 = masses[y];
    let r = positions[y] - positions[x];
    // operation is unoptomized...
    let l = inverseSqrt(r.x * r.x + r.y * r.y + r.z * r.z);
    let f = m1 * m2 * G * l * l * l  * r;
    let a = f * D / m1;

    velocities[x] += a;
    velocities[x].w = 0;
//    velocities[x].z = 0;
    
}
