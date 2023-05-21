use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets, render_resource::*, renderer::RenderDevice,
        texture::TextureCache, Extract, RenderApp, RenderSet,
    },
};

use crate::vox::VoxLoader;

#[derive(Default)]
pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VoxLoader>()
            .add_plugin(MaterialPlugin::<VoxelMaterial>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "8dd2b425-45a2-4a53-ac29-7ce356b2d5fe"]
pub struct VoxelMaterial {
    pub vox_texture: Handle<Image>,
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
        let texture = images.get(&self.vox_texture).unwrap();

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "voxel_material_bind_group".into(),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture.texture_view),
            }],
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
                // @group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
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
            ],
        })
    }
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        r#"C:\Users\dylan\dev\lastattempt\assets\shaders\voxel_material.wgsl"#.into()
    }
}
