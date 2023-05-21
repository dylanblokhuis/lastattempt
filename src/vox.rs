use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::Image,
    reflect::TypeUuid,
    render::render_resource::Extent3d,
    utils::BoxedFuture,
};
use dot_vox::DotVoxData;

#[derive(Default)]
pub struct VoxLoader;

impl AssetLoader for VoxLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(load_vox(bytes, load_context))
    }

    fn extensions(&self) -> &[&str] {
        &["vox"]
    }
}

async fn load_vox<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> anyhow::Result<()> {
    let data: DotVoxData = match dot_vox::load_bytes(bytes) {
        Ok(d) => d,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to load vox file: {}", e));
        }
    };

    let model = data.models.first().unwrap();
    let extent = Extent3d {
        width: model.size.x + 2,
        height: model.size.z + 2,
        depth_or_array_layers: model.size.y + 2,
    };

    let mut vox_bytes: Vec<u8> =
        vec![0; (extent.width * extent.height * extent.depth_or_array_layers) as usize];

    for voxel in &model.voxels {
        let index = (voxel.x as u32 + 1)
            + (voxel.z as u32 + 1) * extent.width
            + (voxel.y as u32 + 1) * extent.width * extent.height;

        vox_bytes[index as usize] = voxel.i;
    }

    load_context.set_default_asset(LoadedAsset::new(Image::new(
        extent,
        bevy::render::render_resource::TextureDimension::D3,
        vox_bytes,
        bevy::render::render_resource::TextureFormat::R8Uint,
    )));

    Ok(())
}
