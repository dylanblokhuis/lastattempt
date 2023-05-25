use std::sync::Arc;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_3d::graph::node::{TONEMAPPING, UPSCALING},
        tonemapping::TonemappingNode,
        upscaling::UpscalingNode,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::*,
        Render, RenderApp, RenderSet,
    },
};
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::{BHShape, BoundingHierarchy},
};

use crate::{
    vox_gi::{
        node::VoxelGINode,
        pipeline::{prepare_pipelines, VoxelGIPipeline},
    },
    vox_plugin::VoxelMaterial,
};

use super::pipeline::VoxelGI;

pub const VOXEL_GI_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3717171717171755);
pub const VOXEL_GI_GRAPH: &str = "voxel_gi_graph";
pub const VOXEL_GI_NODE: &str = "voxel_gi_node";

#[derive(Default)]
pub struct VoxelGIPlugin;

impl Plugin for VoxelGIPlugin {
    fn build(&self, _app: &mut App) {}
    fn finish(&self, app: &mut App) {
        load_internal_asset!(app, VOXEL_GI_SHADER, "voxel_gi.wgsl", Shader::from_wgsl);

        app.add_plugin(ExtractComponentPlugin::<VoxelGI>::default())
            .add_systems(Update, construct_vox_octree);

        let render_app = &mut app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<VoxelGIPipeline>()
            .init_resource::<SpecializedComputePipelines<VoxelGIPipeline>>()
            .add_systems(Render, (prepare_pipelines).in_set(RenderSet::Prepare))
            .add_render_sub_graph(VOXEL_GI_GRAPH)
            .add_render_graph_node::<ViewNodeRunner<VoxelGINode>>(VOXEL_GI_GRAPH, VOXEL_GI_NODE)
            .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(VOXEL_GI_GRAPH, TONEMAPPING)
            .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(VOXEL_GI_GRAPH, UPSCALING)
            .add_render_graph_edges(VOXEL_GI_GRAPH, &[VOXEL_GI_NODE, TONEMAPPING, UPSCALING]);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelBox {
    pub pos: Vec3,
    pub scale: Vec3,
    node_index: usize,
    pub voxels: [u32; 512],
}

impl Bounded for VoxelBox {
    fn aabb(&self) -> AABB {
        let min = self.pos + Vec3::new(-self.scale.x, -self.scale.y, -self.scale.z);
        let max = self.pos + Vec3::new(self.scale.x, self.scale.y, self.scale.z);
        AABB::with_bounds(min.to_array().into(), max.to_array().into())
    }
}
impl BHShape for VoxelBox {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }
    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

#[derive(Copy, Clone, ShaderType, Default, Debug)]
pub struct GpuFlatNode {
    aabb_min: Vec3,
    aabb_max: Vec3,
    entry_index: u32,
    exit_index: u32,

    /// The index of the shape in the shapes array.
    shape_index: u32,
}
#[derive(ShaderType, Default)]
pub struct GpuBVH {
    length: u32,
    #[size(runtime)]
    pub data: Vec<GpuFlatNode>,
}

#[derive(Copy, Clone, ShaderType, Debug)]
#[repr(C, align(16))]
pub struct GpuShape {
    aabb_min: Vec3,
    aabb_max: Vec3,
    _pad: Vec2,
    voxels: [u32; 512],
}

#[derive(ShaderType, Default)]
pub struct GpuShapes {
    length: u32,
    #[size(runtime)]
    pub data: Vec<GpuShape>,
}

fn construct_vox_octree(
    vox_assets: ResMut<Assets<VoxelMaterial>>,
    assets: ResMut<Assets<Image>>,
    entities: Query<(&Transform, &Handle<VoxelMaterial>), With<Handle<VoxelMaterial>>>,
    render_device: Res<bevy::render::renderer::RenderDevice>,
    render_queue: Res<bevy::render::renderer::RenderQueue>,
    mut views: Query<&mut VoxelGI>,
) {
    for mut voxel_bvh in views.iter_mut() {
        if voxel_bvh.is_updated {
            return;
        }

        let mut capacity_needed = 0;
        for (_, mat) in entities.iter() {
            let mat = vox_assets.get(mat).unwrap();
            let Some(model_texture) = mat.model_texture.as_ref() else {
                continue;
            };
            let texture = assets.get(model_texture).unwrap();
            capacity_needed += texture.texture_descriptor.size.width
                * texture.texture_descriptor.size.height
                * texture.texture_descriptor.size.depth_or_array_layers;
        }

        let mut shapes = Vec::with_capacity(capacity_needed as usize);

        for (transform, mat) in entities.iter() {
            let mat = vox_assets.get(mat).unwrap();
            let Some(model_texture) = mat.model_texture.as_ref() else {
                continue;
            };
            let Some(palette_texture) = mat.palette_texture.as_ref() else {
                continue;
            };
            let texture = assets.get(model_texture).unwrap();
            let palette_texture = assets.get(palette_texture).unwrap();
            let rgba_data: Vec<[u8; 4]> = palette_texture
                .data
                .chunks(4)
                .map(|c| c.try_into().unwrap())
                .collect();

            // let voxels = vec![];
            // for x in 0..texture.texture_descriptor.size.width {
            //     for y in 0..texture.texture_descriptor.size.height {
            //         for z in 0..texture.texture_descriptor.size.depth_or_array_layers {
            //             let texture_index = (x
            //                 + y * texture.texture_descriptor.size.width
            //                 + z * texture.texture_descriptor.size.width
            //                     * texture.texture_descriptor.size.height)
            //                 as usize;

            //             let palette_index = texture.data[texture_index];
            //             if palette_index == 0 {
            //                 continue;
            //             }
            //             let scale = Vec3::new(
            //                 texture.texture_descriptor.size.width as f32,
            //                 texture.texture_descriptor.size.height as f32,
            //                 texture.texture_descriptor.size.depth_or_array_layers as f32,
            //             ) / 26.0;
            //             let half_scale = scale / 2.0;
            //             voxels.push((
            //                 Vec3::new(x as f32, y as f32, z as f32) - transform.translation,
            //                 palette_index,
            //             ));
            //         }
            //     }
            // }

            let chunk_size = 8;

            for chunk_x in (0..texture.texture_descriptor.size.width).step_by(chunk_size) {
                for chunk_y in (0..texture.texture_descriptor.size.height).step_by(chunk_size) {
                    for chunk_z in (0..texture.texture_descriptor.size.depth_or_array_layers)
                        .step_by(chunk_size)
                    {
                        let mut voxels: [u32; 512] = [0; 512];
                        let mut voxel_index = 0;

                        for x in chunk_x
                            ..(chunk_x + chunk_size as u32)
                                .min(texture.texture_descriptor.size.width)
                        {
                            for y in chunk_y
                                ..(chunk_y + chunk_size as u32)
                                    .min(texture.texture_descriptor.size.height)
                            {
                                for z in chunk_z
                                    ..(chunk_z + chunk_size as u32)
                                        .min(texture.texture_descriptor.size.depth_or_array_layers)
                                {
                                    // Here you should extract the voxel data from your texture at position (x, y, z)
                                    // and assign it to voxels[voxel_index]
                                    // for example:
                                    voxels[voxel_index] = texture.data[(x
                                        + y * texture.texture_descriptor.size.width
                                        + z * texture.texture_descriptor.size.width
                                            * texture.texture_descriptor.size.height)
                                        as usize]
                                        as u32;

                                    voxel_index += 1;
                                }
                            }
                        }

                        let shape = VoxelBox {
                            scale: Vec3::new(0.5, 0.5, 0.5),
                            pos: transform.translation
                                + Vec3::new(chunk_x as f32, chunk_y as f32, chunk_z as f32),
                            voxels,
                            node_index: 0,
                        };
                        shapes.push(shape);
                    }
                }
            }

            //     for (i, chunk) in texture.data.chunks(512).enumerate() {
            //         let mut voxels: [u32; 512] = [0; 512];

            //         let dim = 8; // Dimension of each side of the voxel chunk

            //         for x in 0..dim {
            //             for y in 0..dim {
            //                 for z in 0..dim {
            //                     let linear_index = x + y * dim + z * dim * dim;

            //                     if linear_index >= chunk.len() {
            //                         break;
            //                     }

            //                     voxels[linear_index] = chunk[linear_index] as u32;
            //                 }
            //             }
            //         }

            //         let shape = VoxelBox {
            //             scale: Vec3::new(0.5, 0.5, 0.5),
            //             pos: transform.translation + (i as f32 * 8.0),
            //             voxels,
            //             node_index: 0,
            //         };
            //         shapes.push(shape);
            //     }
        }

        if shapes.is_empty() {
            continue;
        }
        let bvh = bvh::flat_bvh::FlatBVH::build(&mut shapes);
        let gpu_nodes = bvh
            .iter()
            .map(|node| GpuFlatNode {
                aabb_min: node.aabb.min.to_array().into(),
                aabb_max: node.aabb.max.to_array().into(),
                entry_index: node.entry_index,
                exit_index: node.exit_index,
                shape_index: node.shape_index,
            })
            .collect::<Vec<_>>();

        println!("Voxel BVH: {}", gpu_nodes.len());
        let gpu_bvh = GpuBVH {
            length: gpu_nodes.len() as u32,
            data: gpu_nodes,
        };

        let mut bvh = StorageBuffer::<GpuBVH>::default();
        bvh.set(gpu_bvh);
        bvh.set_label(Some("Voxel BVH"));
        bvh.write_buffer(&render_device, &render_queue);

        let mut shapes_buf = StorageBuffer::<GpuShapes>::default();
        shapes_buf.set(GpuShapes {
            length: shapes.len() as u32,
            data: shapes
                .iter()
                .map(|shape| GpuShape {
                    aabb_min: shape.aabb().min.to_array().into(),
                    aabb_max: shape.aabb().max.to_array().into(),
                    _pad: Vec2::default(),
                    voxels: shape.voxels,
                })
                .collect::<Vec<_>>(),
        });
        shapes_buf.set_label(Some("Voxel BVH Shapes"));
        shapes_buf.write_buffer(&render_device, &render_queue);

        voxel_bvh.bvh = Arc::new(bvh);
        voxel_bvh.shapes = Arc::new(shapes_buf);
        voxel_bvh.is_updated = true;
    }
}
