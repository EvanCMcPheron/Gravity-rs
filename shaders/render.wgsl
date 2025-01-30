
@vertex
fn vs_main(@location(0) vertex: vec4f) -> @builtin(position) vec4f {
    // to future evan - the shdaer is receiving the vertices but for some reason the value is no longer on screen
    // I would guess this is because the CPU and GPU have differnt byte orders, and so the vertex buffer has to be reversed
    return vertex;
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    return vec4f(1., 1., 1., 1.);
}
