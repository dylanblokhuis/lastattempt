use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::{shape, Handle, Image, Mesh},
    reflect::TypeUuid,
    render::render_resource::Extent3d,
    utils::BoxedFuture,
};
use dot_vox::{DotVoxData, Model};

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

#[derive(Debug, TypeUuid)]
#[uuid = "3e859aec-95e6-4aca-bf50-91a6fecdcedd"]
pub struct Vox {
    pub model_texture: Handle<Image>,
    pub palette_texture: Handle<Image>,
    pub mesh: Handle<Mesh>,
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
    let model_texture = get_model_texture(model);
    let mesh =
        load_context.set_labeled_asset("mesh", LoadedAsset::new(get_mesh_from_model(&model)));
    let model = load_context.set_labeled_asset("model", LoadedAsset::new(model_texture));
    let palette = load_context.set_labeled_asset(
        "palette",
        LoadedAsset::new(get_palette_texture(data.palette)),
    );

    load_context.set_default_asset(LoadedAsset::new(Vox {
        model_texture: model,
        palette_texture: palette,
        mesh,
    }));
    Ok(())
}

pub fn get_model_texture(model: &Model) -> Image {
    // we add a padding of 2 to make sure the raymarcher has enough space
    let extent = Extent3d {
        width: model.size.x,
        height: model.size.z,
        depth_or_array_layers: model.size.y,
    };

    let mut vox_bytes: Vec<u8> =
        vec![0; (extent.width * extent.height * extent.depth_or_array_layers) as usize];

    for voxel in &model.voxels {
        let index = (voxel.x as u32)
            + (voxel.z as u32) * extent.width
            + (voxel.y as u32) * extent.width * extent.height;

        vox_bytes[index as usize] = voxel.i + 1;
    }

    Image::new(
        extent,
        bevy::render::render_resource::TextureDimension::D3,
        vox_bytes,
        bevy::render::render_resource::TextureFormat::R8Uint,
    )
}

pub fn get_palette_texture(palette: Vec<dot_vox::Color>) -> Image {
    let mut palette = palette;
    palette.insert(
        0,
        dot_vox::Color {
            a: 0,
            r: 0,
            g: 0,
            b: 0,
        },
    );

    Image::new(
        Extent3d {
            width: palette.len() as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D1,
        palette
            .iter()
            .flat_map(|color| vec![color.r, color.g, color.b, color.a])
            .collect::<Vec<_>>(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
    )
}

pub fn get_mesh_from_model(model: &Model) -> Mesh {
    Mesh::from(shape::Box::new(
        model.size.x as f32 / 4.0,
        model.size.z as f32 / 4.0,
        model.size.y as f32 / 4.0,
    ))
}
