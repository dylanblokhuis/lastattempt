use bevy::render::{
    render_resource::*,
    renderer::RenderDevice,
    view::{ViewTarget, ViewUniform, ViewUniforms},
};

use super::{
    pipeline::{VoxelGI, VoxelGIPipeline},
    plugin::{GpuBVH, GpuShapes},
};

pub fn create_view_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("voxel_gi_view_bind_group_layout"),
        entries: &[
            // View uniforms
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            },
            // Output texture
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadWrite,
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    })
}

pub fn create_view_bind_group(
    view_uniforms: &ViewUniforms,
    view_target: &ViewTarget,
    pipeline: &VoxelGIPipeline,
    render_device: &RenderDevice,
) -> Option<BindGroup> {
    view_uniforms.uniforms.binding().map(|view_uniforms| {
        render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("voxel_gi_view_bind_group"),
            layout: &pipeline.view_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: view_uniforms.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(view_target.main_texture()),
                },
            ],
        })
    })
}

pub fn create_scene_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("voxel_gi_scene_bind_group_layout"),
        entries: &[
            // bvh
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(GpuBVH::min_size()),
                },
                count: None,
            },
            // bvh shapes
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(GpuShapes::min_size()),
                },
                count: None,
            },
        ],
    })
}

pub fn create_scene_bind_group(
    render_device: &RenderDevice,
    pipeline: &VoxelGIPipeline,
    voxel_gi: &VoxelGI,
) -> Option<BindGroup> {
    let Some(bvh_buf) = voxel_gi.bvh.binding() else {
        return None;
    };
    let Some(shapes_buf) = voxel_gi.shapes.binding() else {
        return None;
    };
    Some(render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("voxel_gi_scene_bind_group"),
        layout: &pipeline.scene_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: bvh_buf,
            },
            BindGroupEntry {
                binding: 1,
                resource: shapes_buf,
            },
        ],
    }))
}
