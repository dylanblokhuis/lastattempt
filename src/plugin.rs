use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets, render_resource::*, renderer::RenderDevice,
        texture::TextureCache, Extract, RenderApp, RenderSet,
    },
};

use crate::vox::{self, Vox, VoxLoader};

#[derive(Default)]
pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VoxLoader>()
            .add_asset::<Vox>()
            .add_plugin(MaterialPlugin::<VoxelMaterial>::default())
            .add_system(load_material_textures);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
    }
}

fn load_material_textures(
    mut commands: Commands,
    mut vox_materials: ResMut<Assets<VoxelMaterial>>,
    vox_assets: ResMut<Assets<Vox>>,
    mesh_assets: ResMut<Assets<Mesh>>,
    entities: Query<(Entity, &Handle<VoxelMaterial>), With<Handle<VoxelMaterial>>>,
) {
    for (mat_id, material) in vox_materials
        .iter_mut()
        .filter(|(_, material)| material.model_texture.is_none())
    {
        let Some(vox) = vox_assets.get(&material.vox) else {
            continue;
        };
        let Some(mesh) = mesh_assets.get(&vox.mesh) else {
            continue;
        };

        material.model_texture = Some(vox.model_texture.clone());
        material.palette_texture = Some(vox.palette_texture.clone());
        material.voxel_extra_data = VoxelExtraData {
            half_extents: mesh.compute_aabb().unwrap().half_extents.to_array(),
            _padding: 0,
        };
        for (entity, entity_mat_id) in entities.iter() {
            if entity_mat_id.id() != mat_id {
                continue;
            }
            commands.entity(entity).insert(vox.mesh.clone());
        }
    }
}

#[derive(Debug, Clone, TypeUuid, Default)]
#[uuid = "8dd2b425-45a2-4a53-ac29-7ce356b2d5fe"]
pub struct VoxelMaterial {
    pub vox: Handle<Vox>,
    /// R8Uint texture containing the voxel data
    pub model_texture: Option<Handle<Image>>,
    /// Srgb texture containing the palette data
    pub palette_texture: Option<Handle<Image>>,
    pub voxel_extra_data: VoxelExtraData,
}

#[derive(Debug, Clone, Default, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct VoxelExtraData {
    pub half_extents: [f32; 3],
    pub _padding: u32,
}

#[derive(Bundle, Clone, Default)]
pub struct VoxelBundle {
    pub material: Handle<VoxelMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

impl AsBindGroup for VoxelMaterial {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &bevy::render::render_resource::BindGroupLayout,
        render_device: &bevy::render::renderer::RenderDevice,
        images: &bevy::render::render_asset::RenderAssets<Image>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<
        bevy::render::render_resource::PreparedBindGroup<Self::Data>,
        bevy::render::render_resource::AsBindGroupError,
    > {
        let model = if let Some(ref model_texture) = &self.model_texture {
            images.get(model_texture).unwrap()
        } else {
            return Err(AsBindGroupError::RetryNextUpdate);
        };
        let palette = if let Some(ref palette_texture) = &self.palette_texture {
            images.get(palette_texture).unwrap()
        } else {
            return Err(AsBindGroupError::RetryNextUpdate);
        };

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[self.voxel_extra_data]),
            label: Some("voxel_extra_data_buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "voxel_material_bind_group".into(),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&model.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&palette.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                },
            ],
        });

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn bind_group_layout(
        render_device: &bevy::render::renderer::RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout
    where
        Self: Sized,
    {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: "voxel_material_layout".into(),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Uint,
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}

impl Material for VoxelMaterial {
    fn prepass_fragment_shader() -> ShaderRef {
        r#"C:\Users\dylan\dev\lastattempt\assets\shaders\voxel_material_prepass.wgsl"#.into()
    }
    fn fragment_shader() -> ShaderRef {
        r#"C:\Users\dylan\dev\lastattempt\assets\shaders\voxel_material.wgsl"#.into()
    }

    fn specialize(
        pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}
