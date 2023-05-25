#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::pbr_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions

#import bevy_pbr::prepass_utils

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
    #import bevy_pbr::mesh_vertex_output
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
    @location(0) color: vec4<f32>,
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
    var hit_point = pnt;
    pnt = (pnt - bounding_box_min) / (bounding_box_max - bounding_box_min) * vec3<f32>(count_voxels);

    // epsilon
    var map_pos = vec3<i32>(pnt + 0.0001);
    let delta_dist = abs(vec3(length(direction)) / direction);
    let ray_dir_sign = sign(direction);
    let ray_step = vec3<i32>(ray_dir_sign);
    var side_dist = (ray_dir_sign * (vec3<f32>(map_pos) - pnt) + (ray_dir_sign * 0.5) + 0.5) * delta_dist;
    var mask = vec3(false, false, false);

    var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let zero = vec3<i32>(0);
    let max_voxels = vec3<i32>(count_voxels);
    let max_either_axis = (max(max(max_voxels.x, max_voxels.y), max_voxels.z) * 2);

    var hit = false;

    for (var i: i32 = 0; i < max_either_axis; i = i + 1) {
        let voxel = textureLoad(model_texture, map_pos, 0).r;

        if voxel != u32(0) {
            hit = true;
            let color = textureLoad(palette_texture, i32(voxel), 0).rgb;
            var hit_color = color;
            var normal = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            if mask.x {
                hit_color = hit_color * vec3<f32>(0.5, 0.5, 0.5);
                normal = -ray_dir_sign.x * vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
            if mask.y {
                hit_color = hit_color * vec3<f32>(1.0, 1.0, 1.0);
                normal = -ray_dir_sign.y * vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }
            if mask.z {
                hit_color = hit_color * vec3<f32>(0.75, 0.75, 0.75);
                normal = -ray_dir_sign.z * vec4<f32>(0.0, 0.0, 1.0, 1.0);
            }
            
            

            final_color = vec4<f32>(hit_color, 1.0);

            // var pbr_input: PbrInput = pbr_input_new();
            // pbr_input.material.base_color = vec4<f32>(final_color.rgb, 1.0);
            // pbr_input.frag_coord = in.frag_coord;
            // pbr_input.world_position = vec4<f32>(pnt, 1.0);
            // pbr_input.world_normal =  mesh_normal_local_to_world(normal.rgb);
            // pbr_input.N = mesh_normal_local_to_world(normal.rgb);
            // pbr_input.V = -direction;

            out.color = final_color;
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