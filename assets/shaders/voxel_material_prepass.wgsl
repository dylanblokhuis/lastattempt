#import bevy_pbr::prepass_bindings

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
    @location(0) world_position: vec4<f32>,
};

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

struct FragmentOutput {
    @location(0) normal: vec4<f32>,
    @location(1) motion_vector: vec2<f32>,
}

fn point_to_depth(position: vec3<f32>) -> f32 {
    let pos_in_clip_space = view.view_proj * vec4(position, 1.0);
    let depth_in_fb = (pos_in_clip_space.z / pos_in_clip_space.w);
    return depth_in_fb;
}

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    var count_voxels = vec3<i32>(textureDimensions(model_texture, 0).xyz);

    var inverse_model = transpose(mesh.inverse_transpose_model);
    var local_dir: vec4<f32> = inverse_model * vec4(normalize(in.world_position.xyz - view.world_position.xyz), 0.0);
    var local_orig = inverse_model * vec4(view.world_position.xyz, 1.0);

    var pnt = local_orig.xyz;
    let direction: vec3<f32> = local_dir.xyz;

    let bounding_box_min = vec3<f32>(-voxel_extra_data.half_extents / VOXEL_SCALE);
    let bounding_box_max = vec3<f32>(voxel_extra_data.half_extents / VOXEL_SCALE);

    pnt = pnt + direction * max(0.0, intersect_aabb(pnt, direction, bounding_box_min, bounding_box_max).x);
    pnt = (pnt - bounding_box_min) / (bounding_box_max - bounding_box_min) * vec3<f32>(count_voxels);

    var map_pos = vec3<i32>(pnt);
    let delta_dist = abs(vec3(length(direction)) / direction);
    let ray_dir_sign = sign(direction);
    let ray_step = vec3<i32>(ray_dir_sign);
    var side_dist = (ray_dir_sign * (vec3<f32>(map_pos) - pnt) + (ray_dir_sign * 0.5) + 0.5) * delta_dist;
    var mask = vec3(false, false, false);
    
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let zero = vec3<i32>(0);
    let max_voxels = vec3<i32>(count_voxels);
    let max_either_axis = (max(max(max_voxels.x, max_voxels.y), max_voxels.z) * 2);

    for (var i: i32 = 0; i < max_either_axis; i = i + 1) {
        let voxel = textureLoad(model_texture, map_pos, 0).r;

        if voxel != u32(0) {
            let color = textureLoad(palette_texture, i32(voxel), 0).rgb;
            var hit_color = color;
            if mask.x {
                hit_color = color * 0.5;
                out.normal = -ray_dir_sign.x * vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
            if mask.y {
                hit_color = color * 0.75;
                out.normal = -ray_dir_sign.y * vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }
            if mask.z {
                hit_color = color;
                out.normal = -ray_dir_sign.z * vec4<f32>(0.0, 0.0, 1.0, 1.0);
            }

            final_color = vec4<f32>(hit_color, 1.0);
            let clip_position_t = view.unjittered_view_proj * in.world_position;
            let clip_position = clip_position_t.xy / clip_position_t.w;
            let previous_clip_position_t = previous_view_proj * in.previous_world_position;
            let previous_clip_position = previous_clip_position_t.xy / previous_clip_position_t.w;
            // These motion vectors are used as offsets to UV positions and are stored
            // in the range -1,1 to allow offsetting from the one corner to the
            // diagonally-opposite corner in UV coordinates, in either direction.
            // A difference between diagonally-opposite corners of clip space is in the
            // range -2,2, so this needs to be scaled by 0.5. And the V direction goes
            // down where clip space y goes up, so y needs to be flipped.
            out.motion_vector = (clip_position - previous_clip_position) * vec2(0.5, -0.5);
            break;
        }
        mask = side_dist.xyz <= min(side_dist.yzx, side_dist.zxy);
        side_dist += vec3<f32>(mask) * delta_dist;
        map_pos += vec3<i32>(mask) * ray_step;

        if (map_pos.x < zero.x || map_pos.y < zero.y || map_pos.z < zero.z) {
            break;
        }
        if (map_pos.x > max_voxels.x || map_pos.y > max_voxels.y || map_pos.z > max_voxels.z) {
            break;
        }
    }

    // if final_color.a == 0.0 {
    //     return ;
    // }


    return out;
}