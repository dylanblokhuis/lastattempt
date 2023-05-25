#import bevy_render::view

struct WorldBvhNode {
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
    entry_index: u32,

    // The index of the `FlatNode` to jump to, if the [`AABB`] test is negative.
    exit_index: u32,

    // The index of the shape in the shapes array.
    shape_index: u32,
}

struct WorldBvh {
    length: u32,
    data: array<WorldBvhNode>,
}

struct WorldVoxel {
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
    pad: vec2<f32>,
    voxels: array<u32, 512>,
}
struct WorldVoxels {
    length: u32,
    data: array<WorldVoxel>,
}

@group(0) @binding(0) 
var<storage, read> world_bvh: WorldBvh;
@group(0) @binding(1) 
var<storage, read> world_shapes: WorldVoxels;

@group(1) @binding(0)
var<uniform> view: View;
@group(1) @binding(1)
var output_texture: texture_storage_2d<rgba16float, read_write>;


fn intersect_aabb(rayOrigin: vec3<f32>, rayDir: vec3<f32>, boxMin: vec3<f32>, boxMax: vec3<f32>) -> vec2<f32> {
    let tMin = (boxMin - rayOrigin) / rayDir;
    let tMax = (boxMax - rayOrigin) / rayDir;

    let t1 = min(tMin, tMax);
    let t2 = max(tMin, tMax);
    let tNear = max(max(t1.x, t1.y), t1.z);
    let tFar = min(min(t2.x, t2.y), t2.z);
    return vec2<f32>(tNear, tFar);
}

fn intersect_aabb_bool(rayOrigin: vec3<f32>, rayDir: vec3<f32>, boxMin: vec3<f32>, boxMax: vec3<f32>) -> bool {
    let tMin = (boxMin - rayOrigin) / rayDir;
    let tMax = (boxMax - rayOrigin) / rayDir;

    let t1 = min(tMin, tMax);
    let t2 = max(tMin, tMax);
    let tNear = max(max(t1.x, t1.y), t1.z);
    let tFar = min(min(t2.x, t2.y), t2.z);
    return max(tNear, 0.0) <= tFar;
}
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

fn traverse_voxels(ray: Ray, shape: WorldVoxel, dist: vec2<f32>) -> vec4<f32> {
    let count_voxels = vec3<f32>(8.0);
    var pnt = ray.origin.xyz;
    let direction: vec3<f32> = ray.direction.xyz;

    let bounding_box_min = shape.aabb_min;
    let bounding_box_max = shape.aabb_max;

    pnt = pnt + direction * dist.x;
    var hit_point = pnt;
    pnt = (pnt - bounding_box_min) / (bounding_box_max - bounding_box_min) * vec3<f32>(count_voxels);

    var map_pos = vec3<i32>(pnt);
    let delta_dist = abs(vec3(length(direction)) / direction);
    let ray_dir_sign = sign(direction);
    let ray_step = vec3<i32>(ray_dir_sign);
    var side_dist = (ray_dir_sign * (vec3<f32>(map_pos) - pnt) + (ray_dir_sign * 0.5) + 0.5) * delta_dist;
    var mask = vec3(false, false, false);

    // var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let zero = vec3<i32>(0);
    let max_voxels = vec3<i32>(count_voxels);
    let max_either_axis = (max(max(max_voxels.x, max_voxels.y), max_voxels.z) * 2);

    var hit = false;

    var local_shape = shape;
    for (var i: i32 = 0; i < max_either_axis; i = i + 1) {
        let voxel = local_shape.voxels[map_pos.x + map_pos.y * max_voxels.x + map_pos.z * max_voxels.x * max_voxels.y];

        if voxel != u32(0) {
            hit = true;
            // let color = textureLoad(palette_texture, i32(voxel), 0).rgb;
            // var hit_color = color;
            // var normal = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            // if mask.x {
            //     normal = -ray_dir_sign.x * vec4<f32>(1.0, 0.0, 0.0, 1.0);
            // }
            // if mask.y {
            //     normal = -ray_dir_sign.y * vec4<f32>(0.0, 1.0, 0.0, 1.0);
            // }
            // if mask.z {
            //     normal = -ray_dir_sign.z * vec4<f32>(0.0, 0.0, 1.0, 1.0);
            // }

            // final_color = vec4<f32>(hit_color, 1.0);

            // var pbr_input: PbrInput = pbr_input_new();
            // pbr_input.material.base_color = vec4<f32>(final_color.rgb, 1.0);
            // pbr_input.frag_coord = in.frag_coord;
            // pbr_input.world_position = vec4<f32>(pnt, 1.0);
            // pbr_input.world_normal =  mesh_normal_local_to_world(normal.rgb);
            // pbr_input.N = mesh_normal_local_to_world(normal.rgb);
            // pbr_input.V = -direction;

            // out.color = pbr(pbr_input);
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
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

    return vec4<f32>(0.0);
}

fn traverse_bvh(ray: Ray) -> vec4<f32> {
    var node_index = u32(0);

    var closest_result = vec4<f32>(0.0);
    var latest_dist = -1.0;
    let color = vec4<f32>(0.0);

    while node_index < world_bvh.length {
        let node = world_bvh.data[node_index];

        if node.entry_index == u32(-1) {
            var shape = world_shapes.data[node.shape_index];
            let dist = intersect_aabb(ray.origin, ray.direction, shape.aabb_min, shape.aabb_max);
            if dist.x < dist.y {
                return traverse_voxels(ray, shape, dist);
            }

            node_index = node.exit_index;
        } else if intersect_aabb_bool(ray.origin, ray.direction, node.aabb_min, node.aabb_max) {
            node_index = node.entry_index;
        } else {
            node_index = node.exit_index;
        }
    }

    return closest_result;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let pixel_center = vec2<f32>(global_id.xy) + 0.5;
    let pixel_uv = pixel_center / view.viewport.zw;
    let pixel_ndc = (pixel_uv * 2.0) - 1.0;
    let primary_ray_target = view.inverse_view_proj * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    var ray_origin = view.world_position;
    var ray_direction = normalize((primary_ray_target.xyz / primary_ray_target.w) - ray_origin);

    // let ray_origin = view.world_position.xyz;
    // let ray_direction = normalize(world_space_pos.xyz - ray_origin);
    let color = traverse_bvh(Ray(ray_origin, ray_direction));

    textureStore(output_texture, global_id.xy, color);
}