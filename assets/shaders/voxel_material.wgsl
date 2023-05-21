#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

@group(1) @binding(0)
var texture: texture_3d<u32>;

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

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var count_voxels = vec3<i32>(textureDimensions(texture, 0).xyz);

    var inverse_model = transpose(mesh.inverse_transpose_model);
    var V: vec3<f32> = normalize(in.world_position.xyz - view.world_position.xyz);
    var local_dir: vec4<f32> = inverse_model * vec4(V, 0.0);
    var local_orig = inverse_model * vec4(view.world_position.xyz, 1.0);

    var pnt = local_orig.xyz;
    let direction: vec3<f32> = local_dir.xyz;

    let bounding_box_min = vec3<f32>(-0.5);
    let bounding_box_max = vec3<f32>(0.5);

    pnt = pnt + direction * max(0.0, intersect_aabb(pnt, direction, bounding_box_min, bounding_box_max).x);
    pnt = (pnt + bounding_box_max) * vec3<f32>(count_voxels) * vec3<f32>(count_voxels);


    var map_pos = vec3<i32>(pnt);
    let delta_dist = abs(vec3(length(direction)) / direction);
    let ray_dir_sign = sign(direction);
    let ray_step = vec3<i32>(ray_dir_sign);
    var side_dist = (ray_dir_sign * (vec3<f32>(map_pos) - pnt) + (ray_dir_sign * 0.5) + 0.5) * delta_dist;
    var mask = vec3(false, false, false);

    
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
 
    for (var i: i32 = 0; i < 1000; i = i + 1) {
        let voxel = textureLoad(texture, map_pos / vec3<i32>(count_voxels), 0).r;

        if voxel != u32(0) {
            let color = vec3<f32>(1.0, 0.0, 0.0);
            var hit_color = color;
            if mask.x {
                hit_color = color * 0.5;
            }
            if mask.y {
                hit_color = color * 0.75;
            }
            if mask.z {
                hit_color = color;
            }

            final_color = vec4<f32>(hit_color, 1.0);
            break;
        }
        mask = side_dist.xyz <= min(side_dist.yzx, side_dist.zxy);
        side_dist += vec3<f32>(mask) * delta_dist;
        map_pos += vec3<i32>(mask) * ray_step;

        
    }

    // if all(final_color == vec4<f32>(0.0, 0.0, 0.0, 0.0)) {
    //     discard;
    // }

    return final_color;
}