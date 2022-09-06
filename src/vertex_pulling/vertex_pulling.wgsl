struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    width: f32,
    height: f32,
};

struct ClippingPlaneRange {
    origin: vec3<f32>,
    unit_normal: vec3<f32>,
    min_sdist: f32,
    max_sdist: f32,
};

struct ClippingPlaneRanges {
    ranges: array<ClippingPlaneRange, 3>,
    num_ranges: u32,
}

struct Cuboid {
    min: vec3<f32>,
    meta_bits: u32,
    max: vec3<f32>,
    color: u32,
};

struct Cuboids {
    data: array<Cuboid>,
};

struct Transform {
    m: mat4x4<f32>,
    m_inv: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var<uniform> clipping_planes: ClippingPlaneRanges;

@group(2) @binding(0)
var<uniform> transform: Transform;

@group(3) @binding(0)
var<storage> cuboids: Cuboids;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,

    #ifdef OUTLINES
    @location(1) face_center_to_corner: vec2<f32>,
    #endif
};

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let instance_index = vertex_index >> 5u;
    let cuboid = cuboids.data[instance_index];

    // Check visibility mask.
    if ((cuboid.meta_bits & 0x01u) != 0u) {
        // Discard this vertex by sending it to zero. This only works because
        // we'll be doing this same culling for every vertex in every triangle
        // in this cuboid.
        return VertexOutput();
    }

    let cuboid_center = (cuboid.min + cuboid.max) / 2.0;

    // Clip any cuboid instance that falls out of the allowed ranges.
    for (var i = 0u; i < clipping_planes.num_ranges; i++) {
        let range = clipping_planes.ranges[i];
        let sdist_to_plane = dot(cuboid_center - range.origin, range.unit_normal);
        if sdist_to_plane < range.min_sdist || sdist_to_plane > range.max_sdist {
            // Discard this vertex by sending it to zero. This only works
            // because we'll be doing this same culling for every vertex in
            // every triangle in this cuboid.
            return VertexOutput();
        }
    }

    // Need to do this calculation in cuboid (model) space so our offsets are grid-aligned.
    var camera_in_cuboid_space = transform.m_inv * vec4<f32>(view.world_position, 1.0);
    camera_in_cuboid_space = camera_in_cuboid_space / camera_in_cuboid_space.w;
    let offset = camera_in_cuboid_space.xyz - cuboid_center;
    let mirror_mask =
        u32(offset.x > 0.0) |
        u32(offset.y > 0.0) << 1u |
        u32(offset.z > 0.0) << 2u;
    let visible_vertex_index = vertex_index ^ mirror_mask;

    let cube_corner = vec3<f32>(
        f32(visible_vertex_index & 0x1u),
        f32((visible_vertex_index & 0x2u) >> 1u),
        f32((visible_vertex_index & 0x4u) >> 2u),
    );
    let model_position = cube_corner * cuboid.max + (1.0 - cube_corner) * cuboid.min;
    let world_position = transform.m * vec4<f32>(model_position, 1.0);

    var out: VertexOutput;
    out.clip_position = view.view_proj * world_position;
    out.color = vec4<f32>(
        f32(cuboid.color & 0xFFu),
        f32((cuboid.color >> 8u) & 0xFFu),
        f32((cuboid.color >> 16u) & 0xFFu),
        f32(cuboid.color >> 24u)
    ) / 255.0;

    // This depth biasing avoids Z-fighting when cuboids have overlapping faces.
    let depth_bias_eps = 0.000004;
    let depth_bias_int = (cuboid.meta_bits >> 8u) & 0xFFu;
    out.clip_position.z *= 1.0 + f32(depth_bias_int) * depth_bias_eps;

    #ifdef OUTLINES

    let centroid_to_corner = 2.0 * (cube_corner - vec3<f32>(0.5));
    let face = (vertex_index >> 3u) & 0x3u;
    if face == 0u {
        out.face_center_to_corner = centroid_to_corner.xy;
    } else if face == 1u {
        out.face_center_to_corner = centroid_to_corner.xz;
    } else {
        out.face_center_to_corner = centroid_to_corner.yz;
    }

    #endif

    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,

    #ifdef OUTLINES
    // "normalized face coordinates" in [-1, 1]^2
    @location(1) face_center_to_fragment: vec2<f32>,
    #endif
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

// Constant-pixel-width edges:
// https://catlikecoding.com/unity/tutorials/advanced-rendering/flat-and-wireframe-shading/

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = in.color;

    #ifdef OUTLINES

    let frag_to_edge = vec2<f32>(1.0) - abs(in.face_center_to_fragment);
    let deltas = fwidth(frag_to_edge);
    let step = smoothstep(vec2<f32>(0.0), deltas, frag_to_edge);
    let min_step = min(step.x, step.y);
    let edge_factor = mix(0.1, 1.0, min_step);
    out.color *= edge_factor;

    #endif

    return out;
}
