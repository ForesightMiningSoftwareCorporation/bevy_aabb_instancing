fn hsl_to_nonlinear_srgb(hue: f32, saturation: f32, lightness: f32) -> vec3<f32> {
    // https://en.wikipedia.org/wiki/HSL_and_HSV#HSL_to_RGB
    let chroma = (1.0 - abs(2.0 * lightness - 1.0)) * saturation;
    let hue_prime = hue / 60.0;
    let largest_component = chroma * (1.0 - abs(hue_prime % 2.0 - 1.0));
    var rgb_temp: vec3<f32>;
    if (hue_prime < 1.0) {
        rgb_temp = vec3<f32>(chroma, largest_component, 0.0);
    } else if (hue_prime < 2.0) {
        rgb_temp = vec3<f32>(largest_component, chroma, 0.0);
    } else if (hue_prime < 3.0) {
        rgb_temp = vec3<f32>(0.0, chroma, largest_component);
    } else if (hue_prime < 4.0) {
        rgb_temp = vec3<f32>(0.0, largest_component, chroma);
    } else if (hue_prime < 5.0) {
        rgb_temp = vec3<f32>(largest_component, 0.0, chroma);
    } else {
        rgb_temp = vec3<f32>(chroma, 0.0, largest_component);
    }
    let lightness_match = lightness - chroma / 2.0;
    return rgb_temp + lightness_match;
}

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
}

struct ScalarHueColorOptions {
    min_visible: f32,
    max_visible: f32,
    clamp_min: f32,
    clamp_max: f32,
    hue_zero: f32,
    hue_slope: f32,
}

struct ColorOptions {
    color_mode: u32,
    wireframe: u32, // Any nonzero value means "on".
    _pad0: u32,
    _pad1: u32,
    scalar_hue: ScalarHueColorOptions,
}

struct ClippingPlaneRange {
    origin: vec3<f32>,
    unit_normal: vec3<f32>,
    min_sdist: f32,
    max_sdist: f32,
}

struct ClippingPlaneRanges {
    ranges: array<ClippingPlaneRange, 16>,
    num_ranges: u32,
}

struct Cuboid {
    min: vec3<f32>,
    meta_bits: u32,
    max: vec3<f32>,
    color: u32,
}

struct Cuboids {
    data: array<Cuboid>,
}

struct Transform {
    m: mat4x4<f32>,
    m_inv: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var<uniform> color_options: ColorOptions;

@group(1) @binding(1)
var<uniform> clipping_planes: ClippingPlaneRanges;

@group(2) @binding(0)
var<uniform> transform: Transform;

@group(3) @binding(0)
var<storage> cuboids: Cuboids;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,

    @location(1) instance_id: u32,

    #ifdef OUTLINES
    @location(2) face_center_to_corner: vec2<f32>,
    #endif
}

fn discard_vertex() -> VertexOutput {
    var out = VertexOutput();
    // Apparently GPUs understand this magic.
    out.clip_position.x = bitcast<f32>(0x7fc00000); // nan
    return out;
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.instance_id = instance_index;

    let cuboid = cuboids.data[instance_index];

    // Check visibility mask.
    if ((cuboid.meta_bits & 0x01u) != 0u) {
        // DISCARD CUBOID
        return discard_vertex();
    }

    if (color_options.color_mode == 1u) {
        // SCALAR HUE
        let opt = color_options.scalar_hue;

        let scalar = bitcast<f32>(cuboid.color);
        if (scalar < opt.min_visible ||
            scalar > opt.max_visible)
        {
            // DISCARD CUBOID
            return discard_vertex();
        }

        // HSL
        let cmin = opt.clamp_min;
        let cmax = opt.clamp_max;
        let s = (clamp(scalar, cmin, cmax) - cmin) / (cmax - cmin);
        let hue = (360.0 + (opt.hue_zero + s * opt.hue_slope)) % 360.0;
        let saturation = 1.0;
        let lightness = 0.5;
        out.color = vec4<f32>(hsl_to_nonlinear_srgb(hue, saturation, lightness), 1.0);
    } else {
        // RGB
        out.color = vec4<f32>(
            f32(cuboid.color & 0xFFu),
            f32((cuboid.color >> 8u) & 0xFFu),
            f32((cuboid.color >> 16u) & 0xFFu),
            255.0
        ) / 255.0;
    }

    let cuboid_center = (cuboid.min + cuboid.max) / 2.0;

    if (clipping_planes.num_ranges > 0u) {
        let tfm_cuboid_center = transform.m * vec4<f32>(cuboid_center, 1.0);
        let tfm_cuboid_center = tfm_cuboid_center.xyz / tfm_cuboid_center.w;

        // Clip any cuboid instance that falls out of the allowed ranges.
        for (var i = 0u; i < clipping_planes.num_ranges; i++) {
            let range = clipping_planes.ranges[i];
            let sdist_to_plane = dot(tfm_cuboid_center - range.origin, range.unit_normal);
            if sdist_to_plane < range.min_sdist || sdist_to_plane > range.max_sdist {
                // DISCARD CUBOID
                return discard_vertex();
            }
        }
    }

    // Need to do this calculation in cuboid (model) space so our offsets are grid-aligned.
    let camera_in_cuboid_space = transform.m_inv * vec4<f32>(view.world_position, 1.0);
    let camera_in_cuboid_space = camera_in_cuboid_space.xyz / camera_in_cuboid_space.w;
    let offset = camera_in_cuboid_space - cuboid_center;
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
    let ndc_position = view.view_proj * world_position;

    out.clip_position = ndc_position;

    // This depth biasing avoids Z-fighting when cuboids have overlapping faces.
    let depth_bias_eps = 0.00000008;
    let depth_bias_int = i32(cuboid.meta_bits >> 16u) - i32(1u << 15u);
    let nudge_z = (ndc_position.z / ndc_position.w) * (1.0 + f32(depth_bias_int) * depth_bias_eps);
    out.clip_position.z = nudge_z * ndc_position.w;

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

    @location(1) instance_id: u32,

    #ifdef OUTLINES
    // "normalized face coordinates" in [-1, 1]^2
    @location(2) face_center_to_fragment: vec2<f32>,
    #endif
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) entity_primitive_id: vec2<u32>,
}

// Constant-pixel-width edges:
// https://catlikecoding.com/unity/tutorials/advanced-rendering/flat-and-wireframe-shading/

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = in.color;
    //out.entity_primitive_id.x = 0;               // entity id
    out.entity_primitive_id.y = in.instance_id;  // primitive id

    #ifdef OUTLINES

    let dist_to_edge = vec2<f32>(1.0) - abs(in.face_center_to_fragment);
    let screen_derivative = fwidth(in.face_center_to_fragment);
    let step = smoothstep(vec2<f32>(0.0), 2.0 * screen_derivative, dist_to_edge);
    let min_step = min(step.x, step.y);

    if color_options.wireframe != 0u {
        let edge_factor = mix(0.0, 1.0, min_step);
        if edge_factor > 0.99999 {
            discard;
        }
    } else {
        let edge_factor = mix(0.5, 1.0, min_step);
        out.color *= edge_factor;
    }

    #endif

    return out;
}
