use bevy::{
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
    mut vox_materials: ResMut<Assets<VoxelMaterial>>,
    vox_assets: ResMut<Assets<Vox>>,
) {
    for (_, material) in vox_materials
        .iter_mut()
        .filter(|(_, material)| material.model_texture.is_none())
    {
        let Some(vox) = vox_assets.get(&material.vox) else {
            continue;
        };
        material.model_texture = Some(vox.model_texture.clone());
        material.palette_texture = Some(vox.palette_texture.clone());
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
            ],
        })
    }
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        r#"C:\Users\dylan\dev\lastattempt\assets\shaders\voxel_material.wgsl"#.into()
    }
}
