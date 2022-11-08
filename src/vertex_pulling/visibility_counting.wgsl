@group(0) @binding(0)
var input_texture: texture_depth_2d;
@group(0) @binding(0)
var output_texture: texture_storage_2d<32float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(output_texture, location, vec3(1.0, 0.0, 0.0));
}
