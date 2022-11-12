@group(0) @binding(0)
var input_texture: texture_depth_2d;
@group(0) @binding(1)
var output_texture: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var s: sampler;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {

    let image_size = vec2(f32(num_workgroups.x * u32(8)), f32(num_workgroups.y * u32(8)));
    let location = vec2<i32>(invocation_id.xy);



    let texel = textureGather(input_texture, s, vec2<f32>(invocation_id.xy) / image_size);
    textureStore(output_texture, location, texel);
}
