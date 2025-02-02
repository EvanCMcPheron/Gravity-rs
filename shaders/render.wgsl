
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
    //let w = inputs.world_mat;
    //let mapped = w * vertex;
    return vertex; //vec4(mapped.xy, 0, 1);
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    return vec4f(1., 1., 1., 1.);
}
