use std::sync::Arc;

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        extract_component::ExtractComponent,
        primitives::Frustum,
        render_resource::*,
        renderer::RenderDevice,
        view::{ColorGrading, VisibleEntities},
    },
};

use super::{
    plugin::{GpuBVH, GpuShapes, VOXEL_GI_GRAPH, VOXEL_GI_SHADER},
    resources::{create_scene_bind_group_layout, create_view_bind_group_layout},
};

#[derive(Resource)]
pub struct VoxelGIPipeline {
    pub view_bind_group_layout: BindGroupLayout,
    pub scene_bind_group_layout: BindGroupLayout,
}

impl FromWorld for VoxelGIPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            view_bind_group_layout: create_view_bind_group_layout(world.resource::<RenderDevice>()),
            scene_bind_group_layout: create_scene_bind_group_layout(
                world.resource::<RenderDevice>(),
            ),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct VoxelGIPipelineKey {}

impl SpecializedComputePipeline for VoxelGIPipeline {
    type Key = VoxelGIPipelineKey;

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("voxel_gi_pipeline".into()),
            layout: vec![
                self.scene_bind_group_layout.clone(),
                self.view_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: VOXEL_GI_SHADER.typed(),
            shader_defs: vec![],
            entry_point: "main".into(),
        }
    }
}

#[derive(Component)]
pub struct VoxelGIPipelineId(pub CachedComputePipelineId);

pub fn prepare_pipelines(
    views: Query<Entity, With<VoxelGI>>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<VoxelGIPipeline>>,
    pipeline: Res<VoxelGIPipeline>,
) {
    for entity in &views {
        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, VoxelGIPipelineKey {});

        commands
            .entity(entity)
            .insert(VoxelGIPipelineId(pipeline_id));
    }
}

#[derive(Component, ExtractComponent, Clone, Default)]
pub struct VoxelGI {
    pub bvh: Arc<StorageBuffer<GpuBVH>>,
    pub shapes: Arc<StorageBuffer<GpuShapes>>,
    pub is_updated: bool,
}

#[derive(Bundle)]
pub struct VoxelGICamera3dBundle {
    pub path_tracer: VoxelGI,
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: Projection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_3d: Camera3d,
    pub tonemapping: Tonemapping,
    pub dither: DebandDither,
    pub color_grading: ColorGrading,
}

impl Default for VoxelGICamera3dBundle {
    fn default() -> Self {
        Self {
            path_tracer: Default::default(),
            camera_render_graph: CameraRenderGraph::new(VOXEL_GI_GRAPH),
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            projection: Default::default(),
            visible_entities: Default::default(),
            frustum: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            camera_3d: Default::default(),
            tonemapping: Tonemapping::ReinhardLuminance,
            dither: DebandDither::Enabled,
            color_grading: ColorGrading::default(),
        }
    }
}
