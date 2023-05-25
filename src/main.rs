use std::{f32::consts::PI, time::Duration};

use basic_camera::{CameraController, CameraControllerPlugin};
use bevy::{
    asset::ChangeWatcher,
    core_pipeline::{
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        tonemapping::Tonemapping,
    },
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{settings::WgpuSettings, RenderPlugin},
};
use vox_gi::{
    pipeline::{VoxelGI, VoxelGICamera3dBundle},
    plugin::VoxelGIPlugin,
};
use vox_plugin::{VoxelBundle, VoxelMaterial, VoxelPlugin};

mod basic_camera;
mod vox;
mod vox_gi;
mod vox_plugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
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
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..Default::default()
        })
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(CameraControllerPlugin)
        .add_plugin(VoxelPlugin)
        // .add_plugin(VoxelGIPlugin)
        .add_plugin(TemporalAntiAliasPlugin)
        .add_systems(Startup, setup)
        // .add_systems(Update, swap_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut vox_materials: ResMut<Assets<VoxelMaterial>>,
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

    let plane = asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\basic-tile.vox"#);

    for x in 0..20 {
        for z in 0..20 {
            commands.spawn(VoxelBundle {
                material: vox_materials.add(VoxelMaterial {
                    vox: plane.clone(),
                    ..Default::default()
                }),
                transform: Transform::from_xyz(-x as f32 * 50.0, -5.0, -z as f32 * 50.0),
                ..Default::default()
            });
        }
    }

    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(1.0, 0.4, 0.4).into()),
        transform: Transform::from_xyz(-100.0, 0.5, 0.0)
            .with_rotation(Quat::from_rotation_y(PI / 4.0)),
        ..Default::default()
    });

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
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                tonemapping: Tonemapping::TonyMcMapface,
                camera: Camera {
                    hdr: true,
                    order: 1,
                    ..Default::default()
                },
                ..default()
            },
            TemporalAntiAliasBundle::default(),
        ))
        .insert(CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        });
}

// fn swap_camera(
//     key_input: Res<Input<KeyCode>>,
//     mut regular_camera: Query<(&mut Camera, &mut Transform), Without<VoxelGI>>,
//     mut voxel_gi_camera: Query<(&mut Camera, &mut Transform), With<VoxelGI>>,
// ) {
//     let (mut camera, mut transform) = regular_camera.single_mut();
//     let (mut vox_camera, mut vox_transform) = voxel_gi_camera.single_mut();

//     transform.translation = vox_transform.translation;
//     transform.rotation = vox_transform.rotation;

//     if key_input.just_pressed(KeyCode::F) {
//         if camera.order == 1 {
//             camera.order = 2;
//             vox_camera.order = 1;
//         } else {
//             camera.order = 1;
//             vox_camera.order = 2;
//         }
//     }
// }
