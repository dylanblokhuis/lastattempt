use std::{f32::consts::PI, time::Duration};

use bevy::{
    core_pipeline::{
        prepass::{DepthPrepass, NormalPrepass},
        tonemapping::Tonemapping,
    },
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{
        primitives::{Aabb, Frustum},
        settings::WgpuSettings,
        RenderPlugin,
    },
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, ParallelSlice, TaskPool},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use noise::{
    core::perlin::{perlin_2d, perlin_3d},
    permutationtable::PermutationTable,
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, NoiseFn, Perlin,
};
use vox::Vox;
use vox_plugin::{VoxelBundle, VoxelMaterial, VoxelPlugin};

use bevy_flycam::prelude::*;

use crate::vox::{get_mesh_from_model, get_model_texture, get_palette_texture};
mod vox;
mod vox_plugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::Mailbox,
                        ..default()
                    }),
                    ..Default::default()
                })
                .set(RenderPlugin {
                    wgpu_settings: WgpuSettings {
                        ..Default::default()
                    },
                }),
        )
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(VoxelPlugin)
        .add_plugin(PlayerPlugin)
        // .add_plugin(VoxelGIPlugin)
        // .add_plugin(TemporalAntiAliasPlugin)
        .add_startup_system(setup)
        .add_system(yo)
        .run();
}

#[derive(Copy, Clone)]
struct SplinePoint {
    continentalness: f64,
    erosion: f64,
    peak_valley: f64,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut vox_materials: ResMut<Assets<VoxelMaterial>>,
    mut vox_assets: ResMut<Assets<Vox>>,
    mut textures: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn(VoxelBundle {
    //     material: vox_materials.add(VoxelMaterial {
    //         vox: asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\3x3x3.vox"#),
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0)
    //         .with_rotation(Quat::from_rotation_y(PI / 4.0)),
    //     ..Default::default()
    // });

    commands.spawn(VoxelBundle {
        material: vox_materials.add(VoxelMaterial {
            vox: asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\castle.vox"#),

            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, -10.0),
        ..Default::default()
    });

    // commands.spawn(VoxelBundle {
    //     material: vox_materials.add(VoxelMaterial {
    //         vox: asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\monu3.vox"#),
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(-20.0, 0.5, -10.0),
    //     ..Default::default()
    // });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(1.0, 0.4, 0.4).into()),
        transform: Transform::from_xyz(-100.0, 0.5, 0.0)
            .with_rotation(Quat::from_rotation_y(PI / 4.0)),
        ..Default::default()
    });

    // let plane = asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\basic-tile.vox"#);

    // for x in 0..20 {
    //     for z in 0..20 {
    //         commands.spawn(VoxelBundle {
    //             material: vox_materials.add(VoxelMaterial {
    //                 vox: plane.clone(),
    //                 ..Default::default()
    //             }),
    //             transform: Transform::from_xyz(-x as f32 * 50.0, -5.0, -z as f32 * 50.0),
    //             ..Default::default()
    //         });
    //     }
    // }

    const CHUNK_WIDTH: usize = 32;
    const CHUNK_HEIGHT: usize = 24;
    const CHUNK_DEPTH: usize = 32;
    let width = 512;
    let height = 24;
    let depth = 512;

    fn generate_noise_map(width: usize, height: usize, depth: usize) -> Vec<f64> {
        let mut perlin = Fbm::<Perlin>::new(234982374);
        perlin.octaves = 2;
        perlin.frequency = 0.5;

        let mut map = Vec::with_capacity(width * height * depth);
        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0, z as f64 / 10.0]);
                    map.push(value);
                }
            }
        }
        map
    }

    fn chunk_noise_map(map: Vec<f64>, width: usize, height: usize, depth: usize) -> Vec<Vec<f64>> {
        let chunks_in_x = width / CHUNK_WIDTH;
        let chunks_in_y = height / CHUNK_HEIGHT;
        let chunks_in_z = depth / CHUNK_DEPTH;

        let mut chunks = Vec::new();

        for zc in 0..chunks_in_z {
            for yc in 0..chunks_in_y {
                for xc in 0..chunks_in_x {
                    let mut chunk = Vec::with_capacity(CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH);

                    for z in 0..CHUNK_DEPTH {
                        for y in 0..CHUNK_HEIGHT {
                            for x in 0..CHUNK_WIDTH {
                                let xi = xc * CHUNK_WIDTH + x;
                                let yi = yc * CHUNK_HEIGHT + y;
                                let zi = zc * CHUNK_DEPTH + z;

                                let value = map[zi * width * height + yi * width + xi];
                                chunk.push(value);
                            }
                        }
                    }

                    chunks.push(chunk);
                }
            }
        }
        chunks
    }

    let map = generate_noise_map(width, height, depth);
    let chunks = chunk_noise_map(map, width, height, depth);

    for (i, chunk) in chunks.iter().enumerate() {
        let mut voxels = vec![];
        for (index, value) in chunk.iter().enumerate() {
            let x = (index / (CHUNK_WIDTH * CHUNK_HEIGHT)) as u8;
            let z = ((index % (CHUNK_WIDTH * CHUNK_HEIGHT)) / CHUNK_WIDTH) as u8;
            let y = ((index % (CHUNK_WIDTH * CHUNK_HEIGHT)) % CHUNK_WIDTH) as u8;

            assert!(x < CHUNK_WIDTH as u8);
            assert!(y < CHUNK_DEPTH as u8);
            assert!(z < CHUNK_HEIGHT as u8);

            if (CHUNK_HEIGHT as u8 - z) < ((value * 50.0) as u8) {
                continue;
            }

            voxels.push(dot_vox::Voxel {
                x: x as u8,
                y: y as u8,
                z: z as u8,
                i: 0,
            });
        }

        println!(
            "voxels: {}, chunk {}",
            voxels.len(),
            CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH
        );
        assert!(voxels.len() <= CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH);

        let model = dot_vox::Model {
            voxels,
            size: dot_vox::Size {
                x: CHUNK_WIDTH as u32,
                y: CHUNK_DEPTH as u32,
                z: CHUNK_HEIGHT as u32,
            },
        };

        let palette = vec![dot_vox::Color {
            r: 50,
            g: 50,
            b: 50,
            a: 255,
        }];

        let chunk_pos = Vec3::new(
            35.0 + ((i % (width / CHUNK_WIDTH)) as f32 * CHUNK_WIDTH as f32) / 4.0,
            0.0,
            35.0 + ((i / (width / CHUNK_WIDTH)) as f32 * CHUNK_WIDTH as f32) / 4.0,
        );

        commands.spawn(VoxelBundle {
            material: vox_materials.add(VoxelMaterial {
                vox: vox_assets.add(Vox {
                    model_texture: textures.add(get_model_texture(&model)),
                    palette_texture: textures.add(get_palette_texture(palette)),
                    mesh: meshes.add(get_mesh_from_model(&model)),
                }),
                ..Default::default()
            }),
            transform: Transform::from_translation(chunk_pos),
            ..Default::default()
        });
    }

    // cube

    // commands
    //     .spawn(VoxelGICamera3dBundle {
    //         transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         camera: Camera {
    //             hdr: true,
    //             order: 2,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .insert(CameraController {
    //         orbit_mode: true,
    //         orbit_focus: Vec3::new(0.0, 0.5, 0.0),
    //         ..default()
    //     });

    // camera
}

fn yo(query: Query<(&ComputedVisibility, Entity), With<Handle<VoxelMaterial>>>) {}
