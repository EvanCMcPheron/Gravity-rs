
struct Uniform {
  world_mat: mat4x4<f32>,
  width: u32,
  height: u32,
  p0_: u32,
  p1_: u32
}

@group(0)
@binding(0)
var<uniform> inputs: Uniform;

@vertex
fn vs_main(@location(0) vertex: vec4f) -> @builtin(position) vec4f {
    let w = inputs.world_mat;
    let mapped = w * vertex;
    return vec4(mapped.xyz / mapped.w, 1);
}

fn map_z(n: f32) -> f32 {
    // Maps the z space from [0,1] -> [0,very large number]
    return (1/pow(1 - 0.9999 * n, 50.)) - 1;
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    return vec4f(position.x, position.y, 1., 1.);
}
