@group(0) @binding(0) var<storage, read_write> positions: array<vec4f>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4f>;
@group(0) @binding(2) var<storage, read_write> masses: array<f32>;

@compute @workgroup_size(1) fn cs_entry(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    
}
