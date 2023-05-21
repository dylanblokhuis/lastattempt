use basic_camera::{CameraController, CameraControllerPlugin};
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_editor_pls::prelude::*;
use plugin::{VoxelBundle, VoxelMaterial, VoxelPlugin};
use vox::Vox;

mod basic_camera;
mod plugin;
mod vox;

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
                        backends: Some(Backends::DX12),
                        ..Default::default()
                    },
                }),
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EditorPlugin::default())
        .add_plugin(CameraControllerPlugin)
        .add_plugin(VoxelPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut vox_materials: ResMut<Assets<VoxelMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(VoxelBundle {
        material: vox_materials.add(
            VoxelMaterial {
                vox: asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\3x3x3.vox"#),
                ..Default::default()
            }
            .into(),
        ),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    commands.spawn(VoxelBundle {
        material: vox_materials.add(
            VoxelMaterial {
                vox: asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\castle.vox"#),
                ..Default::default()
            }
            .into(),
        ),
        transform: Transform::from_xyz(0.0, 0.5, -10.0),
        ..Default::default()
    });

    commands.spawn(VoxelBundle {
        material: vox_materials.add(
            VoxelMaterial {
                vox: asset_server.load(r#"C:\Users\dylan\dev\lastattempt\assets\vox\monu3.vox"#),
                ..Default::default()
            }
            .into(),
        ),
        transform: Transform::from_xyz(-50.0, 0.5, -10.0),
        ..Default::default()
    });

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(1.0, 0.4, 0.4).into()),
        transform: Transform::from_xyz(-2.0, 0.5, 0.0),
        ..Default::default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        })
        .insert(CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        });
}
