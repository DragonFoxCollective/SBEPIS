use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::core_3d::graph::Core3d;
use bevy::ecs::query::QueryItem;
use bevy::platform::collections::HashMap;
use bevy::platform::collections::hash_map::Entry;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
    UniformComponentPlugin,
};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphContext, RenderGraphExt as _, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::binding_types::{
    sampler, storage_buffer, texture_2d, uniform_buffer,
};
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, CachedComputePipelineId,
    CachedRenderPipelineId, ColorTargetState, ColorWrites, ComputePassDescriptor,
    ComputePipelineDescriptor, FragmentState, MultisampleState, Operations, PipelineCache,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, StorageBuffer,
    TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::sync_world::RenderEntity;
use bevy::render::view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::{Extract, Render, RenderApp, RenderSystems};
use bevy_auto_plugin::prelude::*;
use rand::Rng;
use rand::distr::{Distribution, StandardUniform};
use return_ok::{some_or_return, some_or_return_ok};

use crate::post_processing::PostProcessPlugin;

const SHADER_ASSET_PATH: &str = "post_processing_quantize.wgsl";
const KMEANS_SHADER_ASSET_PATH: &str = "post_processing_quantize_post.wgsl";

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = PostProcessPlugin)]
struct PostProcessQuantizePlugin;

impl Plugin for PostProcessQuantizePlugin {
    #[auto_plugin]
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PostProcessQuantizeSettings>::default(),
            UniformComponentPlugin::<PostProcessQuantizeSettings>::default(),
        ));

        let render_app = some_or_return!(app.get_sub_app_mut(RenderApp));

        render_app
            .init_resource::<QuantizeBuffers>()
            .init_resource::<ExtractedQuantizeSettings>()
            .add_systems(ExtractSchedule, extract_buffers)
            .add_systems(Render, prepare_buffers.in_set(RenderSystems::Prepare))
            .add_render_graph_node::<ViewNodeRunner<PostProcessQuantizeNode>>(
                Core3d,
                PostProcessQuantizeLabel,
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = some_or_return!(app.get_sub_app_mut(RenderApp));

        render_app.init_resource::<PostProcessQuantizePipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct PostProcessQuantizeLabel;

#[derive(Default)]
struct PostProcessQuantizeNode;

impl ViewNode for PostProcessQuantizeNode {
    type ViewQuery = (
        &'static ViewUniformOffset,
        &'static ViewTarget,
        &'static DynamicUniformIndex<PostProcessQuantizeSettings>,
    );

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_uniform_offset, view_target, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessQuantizePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let view_uniforms = world.resource::<ViewUniforms>();
        let view_uniforms_binding = some_or_return_ok!(view_uniforms.uniforms.binding());

        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessQuantizeSettings>>();
        let settings_binding = some_or_return_ok!(settings_uniforms.uniforms().binding());

        let quantize_buffers = world.resource::<QuantizeBuffers>();

        let view_entity = graph.view_entity();
        let quantize_buffer = some_or_return_ok!(quantize_buffers.buffers.get(&view_entity));
        let k_buffer = some_or_return_ok!(quantize_buffer.k.binding());
        let clusters_buffer = some_or_return_ok!(quantize_buffer.clusters.binding());
        let furthest_point_buffer = some_or_return_ok!(quantize_buffer.furthest_point.binding());

        let post_process = view_target.post_process_write();

        // Main pass
        {
            let pipeline = some_or_return_ok!(
                pipeline_cache.get_render_pipeline(post_process_pipeline.quantize_pipeline_id)
            );

            let bind_group = render_context.render_device().create_bind_group(
                "quantize_post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    view_uniforms_binding,
                    post_process.source,
                    &post_process_pipeline.sampler,
                    settings_binding.clone(),
                    k_buffer.clone(),
                    clusters_buffer.clone(),
                    furthest_point_buffer.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("quantize_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post_process.destination,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(pipeline);
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        // k-means iteration pass
        {
            let pipeline = some_or_return_ok!(
                pipeline_cache.get_compute_pipeline(post_process_pipeline.kmeans_pipeline_id)
            );

            let bind_group = render_context.render_device().create_bind_group(
                "quantize_kmeans_post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.kmeans_layout),
                &BindGroupEntries::sequential((
                    settings_binding,
                    k_buffer,
                    clusters_buffer,
                    furthest_point_buffer,
                )),
            );

            let mut compute_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("quantize_kmeans_post_process_pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        Ok(())
    }
}

#[auto_resource(plugin = PostProcessQuantizePlugin, derive)]
struct PostProcessQuantizePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    quantize_pipeline_id: CachedRenderPipelineId,
    kmeans_layout: BindGroupLayoutDescriptor,
    kmeans_pipeline_id: CachedComputePipelineId,
}

impl FromWorld for PostProcessQuantizePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = BindGroupLayoutDescriptor::new(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<PostProcessQuantizeSettings>(true),
                    storage_buffer::<u32>(false),
                    storage_buffer::<[QuantizeCluster; 256]>(false),
                    storage_buffer::<FurthestPoint>(false),
                ),
            ),
        );

        let kmeans_layout = BindGroupLayoutDescriptor::new(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<PostProcessQuantizeSettings>(true),
                    storage_buffer::<u32>(false),
                    storage_buffer::<[QuantizeCluster; 256]>(false),
                    storage_buffer::<FurthestPoint>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.load_asset(SHADER_ASSET_PATH);
        let kmeans_shader = world.load_asset(KMEANS_SHADER_ASSET_PATH);

        let fullscreen_shader = world.resource::<FullscreenShader>().clone();

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let quantize_pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("quantize_post_process_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader,
                shader_defs: vec![],
                entry_point: Some("main".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let kmeans_pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("kmeans_post_process_pipeline".into()),
            layout: vec![kmeans_layout.clone()],
            shader: kmeans_shader,
            entry_point: Some("main".into()),
            ..default()
        });

        Self {
            layout,
            sampler,
            quantize_pipeline_id,
            kmeans_layout,
            kmeans_pipeline_id,
        }
    }
}

#[auto_component(plugin = PostProcessQuantizePlugin, derive(Default, Clone, Copy, ExtractComponent, ShaderType), reflect, register)]
pub struct PostProcessQuantizeSettings {
    /// Within `1..=256`. Setting this to `0` means no fixed k.
    pub fixed_k: u32,
}

#[auto_resource(plugin = PostProcessQuantizePlugin, derive(Default))]
struct QuantizeBuffers {
    buffers: HashMap<Entity, QuantizeBuffer>,
}

struct QuantizeBuffer {
    k: StorageBuffer<u32>,
    clusters: StorageBuffer<[QuantizeCluster; 256]>,
    furthest_point: StorageBuffer<FurthestPoint>,
}

#[derive(ShaderType, Clone, Copy)]
struct QuantizeCluster {
    color: Vec3,
    centroid_sum_r: u32,
    centroid_sum_g: u32,
    centroid_sum_b: u32,
    len_centroid_sum: u32,
}

#[derive(ShaderType, Clone, Copy, Default)]
struct FurthestPoint {
    color_r: u32,
    color_g: u32,
    color_b: u32,
    distance: u32,
}

impl Distribution<QuantizeCluster> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuantizeCluster {
        QuantizeCluster {
            color: rng.random(),
            centroid_sum_r: 0,
            centroid_sum_g: 0,
            centroid_sum_b: 0,
            len_centroid_sum: 0,
        }
    }
}

#[auto_resource(plugin = PostProcessQuantizePlugin, derive(Default))]
struct ExtractedQuantizeSettings {
    changed: Vec<(Entity, PostProcessQuantizeSettings)>,
    removed: Vec<Entity>,
}

fn extract_buffers(
    mut commands: Commands,
    changed: Extract<
        Query<(RenderEntity, &PostProcessQuantizeSettings), Changed<PostProcessQuantizeSettings>>,
    >,
    mut removed: Extract<RemovedComponents<PostProcessQuantizeSettings>>,
) {
    commands.insert_resource(ExtractedQuantizeSettings {
        changed: changed
            .iter()
            .map(|(entity, &settings)| (entity, settings))
            .collect(),
        removed: removed.read().collect(),
    });
}

fn prepare_buffers(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut extracted: ResMut<ExtractedQuantizeSettings>,
    mut buffers: ResMut<QuantizeBuffers>,
) {
    for (entity, settings) in extracted.changed.drain(..) {
        match buffers.buffers.entry(entity) {
            Entry::Occupied(mut entry) => {
                let value = entry.get_mut();
                value.k.set(settings.fixed_k);
                value.k.write_buffer(&device, &queue);
            }
            Entry::Vacant(entry) => {
                let initial_clusters = rand::random::<[QuantizeCluster; 256]>();
                let value = entry.insert(QuantizeBuffer {
                    k: settings.fixed_k.into(),
                    clusters: initial_clusters.into(),
                    furthest_point: FurthestPoint::default().into(),
                });
                value.k.write_buffer(&device, &queue);
                value.clusters.write_buffer(&device, &queue);
                value.furthest_point.write_buffer(&device, &queue);
            }
        }
    }

    for entity in extracted.removed.drain(..) {
        buffers.buffers.remove(&entity);
    }
}
