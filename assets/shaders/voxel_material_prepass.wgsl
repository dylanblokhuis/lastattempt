#import bevy_pbr::prepass_bindings
#import bevy_pbr::pbr_bindings
#ifdef NORMAL_PREPASS
#import bevy_pbr::pbr_functions
#endif // NORMAL_PREPASS


struct VoxelExtraData {
    half_extents: vec3<f32>
}

@group(1) @binding(0)
var model_texture: texture_3d<u32>;

@group(1) @binding(1)
var palette_texture: texture_1d<f32>;

@group(1) @binding(2)
var<uniform> voxel_extra_data: VoxelExtraData;


struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
#ifdef VERTEX_UVS
    @location(0) uv: vec2<f32>,
#endif // VERTEX_UVS

    @location(1) world_normal: vec3<f32>,
};

// Cutoff used for the premultiplied alpha modes BLEND and ADD.
const PREMULTIPLIED_ALPHA_CUTOFF = 0.05;

// We can use a simplified version of alpha_discard() here since we only need to handle the alpha_cutoff
fn prepass_alpha_discard(in: FragmentInput) {

// This is a workaround since the preprocessor does not support
// #if defined(ALPHA_MASK) || defined(BLEND_PREMULTIPLIED_ALPHA)
    #ifndef ALPHA_MASK
    #ifndef BLEND_PREMULTIPLIED_ALPHA
    #ifndef BLEND_ALPHA

    #define EMPTY_PREPASS_ALPHA_DISCARD

#endif // BLEND_ALPHA
#endif // BLEND_PREMULTIPLIED_ALPHA not defined
#endif // ALPHA_MASK not defined

    #ifndef EMPTY_PREPASS_ALPHA_DISCARD
    var output_color: vec4<f32> = material.base_color;

#ifdef VERTEX_UVS
    if (material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u {
        output_color = output_color * textureSample(base_color_texture, base_color_sampler, in.uv);
    }
#endif // VERTEX_UVS

#ifdef ALPHA_MASK
    if ((material.flags & STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK) != 0u) && output_color.a < material.alpha_cutoff {
        discard;
    }
#else // BLEND_PREMULTIPLIED_ALPHA || BLEND_ALPHA
    let alpha_mode = material.flags & STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if (alpha_mode == STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND || alpha_mode == STANDARD_MATERIAL_FLAGS_ALPHA_MODE_ADD) && output_color.a < PREMULTIPLIED_ALPHA_CUTOFF {
        discard;
    } else if alpha_mode == STANDARD_MATERIAL_FLAGS_ALPHA_MODE_PREMULTIPLIED && all(output_color < vec4(PREMULTIPLIED_ALPHA_CUTOFF)) {
        discard;
    }
#endif // !ALPHA_MASK

#endif // EMPTY_PREPASS_ALPHA_DISCARD not defined
}

struct FragmentOutput {
#ifdef NORMAL_PREPASS
    @location(0) normal: vec4<f32>,
#endif // NORMAL_PREPASS
}

fn intersect_aabb(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    box_min: vec3<f32>,
    box_max: vec3<f32>
) -> vec2<f32> {
    let t_min = (box_min - ray_origin) / ray_direction;
    let t_max = (box_max - ray_origin) / ray_direction;

    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);

    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);

    return vec2<f32>(t_near, t_far);
}

const VOXEL_SCALE = 1.0;

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    prepass_alpha_discard(in);

    var out: FragmentOutput;

    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if (material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        let world_normal = prepare_world_normal(
            in.world_normal,
            (material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
            in.is_front,
        );

        let normal = apply_normal_mapping(
            material.flags,
            world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
            in.world_tangent,
#endif // STANDARDMATERIAL_NORMAL_MAP
#endif // VERTEX_TANGENTS
#ifdef VERTEX_UVS
            in.uv,
#endif // VERTEX_UVS
        );

        out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);
    } else {
        out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
    }

    var count_voxels = vec3<i32>(textureDimensions(model_texture, 0).xyz);

    var inverse_model = transpose(mesh.inverse_transpose_model);
    var local_dir: vec4<f32> = inverse_model * vec4(normalize(in.world_position.xyz - view.world_position.xyz), 0.0);
    var local_orig = inverse_model * vec4(view.world_position.xyz, 1.0);

    var pnt = local_orig.xyz;
    let direction: vec3<f32> = local_dir.xyz;

    let bounding_box_min = vec3<f32>(-voxel_extra_data.half_extents / VOXEL_SCALE);
    let bounding_box_max = vec3<f32>(voxel_extra_data.half_extents / VOXEL_SCALE);

    pnt = pnt + direction * max(0.0, intersect_aabb(pnt, direction, bounding_box_min, bounding_box_max).x);
    var hit_point = pnt;
    pnt = (pnt - bounding_box_min) / (bounding_box_max - bounding_box_min) * vec3<f32>(count_voxels);

    var map_pos = vec3<i32>(pnt);
    let delta_dist = abs(vec3(length(direction)) / direction);
    let ray_dir_sign = sign(direction);
    let ray_step = vec3<i32>(ray_dir_sign);
    var side_dist = (ray_dir_sign * (vec3<f32>(map_pos) - pnt) + (ray_dir_sign * 0.5) + 0.5) * delta_dist;
    var mask = vec3(false, false, false);

    let zero = vec3<i32>(0);
    let max_voxels = vec3<i32>(count_voxels);
    let max_either_axis = (max(max(max_voxels.x, max_voxels.y), max_voxels.z) * 2);

    var hit = false;

    for (var i: i32 = 0; i < max_either_axis; i = i + 1) {
        let voxel = textureLoad(model_texture, map_pos, 0).r;
        if voxel != u32(0) {
            hit = true;
            break;
        }
        mask = side_dist.xyz <= min(side_dist.yzx, side_dist.zxy);
        side_dist += vec3<f32>(mask) * delta_dist;
        map_pos += vec3<i32>(mask) * ray_step;

        if map_pos.x < zero.x || map_pos.y < zero.y || map_pos.z < zero.z {
            break;
        }
        if map_pos.x > max_voxels.x || map_pos.y > max_voxels.y || map_pos.z > max_voxels.z {
            break;
        }
    }

    if !hit {
        discard;
    }

    return out;
}