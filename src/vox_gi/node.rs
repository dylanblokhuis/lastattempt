use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::{ComputePassDescriptor, PipelineCache},
        renderer::RenderContext,
        view::{ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

use super::{
    pipeline::{VoxelGI, VoxelGIPipeline, VoxelGIPipelineId},
    resources::{create_scene_bind_group, create_view_bind_group},
};

pub struct VoxelGINode(
    QueryState<(
        &'static VoxelGI,
        &'static VoxelGIPipelineId,
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static ExtractedCamera,
    )>,
);

impl ViewNode for VoxelGINode {
    type ViewQuery = (
        &'static VoxelGI,
        &'static VoxelGIPipelineId,
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static ExtractedCamera,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (voxel_gi, pipeline_id, view_target, view_uniform_offset, camera): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (
            Some(pipeline_cache),
            Some(view_uniforms),
            Some(voxel_gi_pipeline),
        ) = (
            world.get_resource::<PipelineCache>(),
            world.get_resource::<ViewUniforms>(),
            world.get_resource::<VoxelGIPipeline>(),
        ) else {
            return Ok(());
        };
        let (Some(pipeline), Some(viewport)) = (pipeline_cache.get_compute_pipeline(pipeline_id.0), camera.physical_viewport_size) else {
            return Ok(());
        };
        let Some(scene_bind_group) = create_scene_bind_group(render_context.render_device(), voxel_gi_pipeline, voxel_gi) else {
            return Ok(());
        };
        let Some(view_bind_group) = create_view_bind_group(view_uniforms,  view_target, voxel_gi_pipeline, render_context.render_device()) else {
            return Ok(());
        };

        {
            let command_encoder = render_context.command_encoder();
            let mut solari_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("voxel_gi_pass"),
            });

            solari_pass.set_pipeline(pipeline);
            solari_pass.set_bind_group(0, &scene_bind_group, &[]);
            solari_pass.set_bind_group(1, &view_bind_group, &[view_uniform_offset.offset]);
            solari_pass.dispatch_workgroups((viewport.x + 7) / 8, (viewport.y + 7) / 8, 1);
        }

        Ok(())
    }

    fn update(&mut self, world: &mut World) {
        self.0.update_archetypes(world);
    }
}

impl FromWorld for VoxelGINode {
    fn from_world(world: &mut World) -> Self {
        Self(QueryState::new(world))
    }
}
