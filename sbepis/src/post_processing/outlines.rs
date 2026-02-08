use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::core_3d::graph::Core3d;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
    UniformComponentPlugin,
};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphContext, RenderGraphExt as _, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::binding_types::{
    sampler, texture_2d, texture_depth_2d, uniform_buffer,
};
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations, PipelineCache,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::{CachedTexture, TextureCache};
use bevy::render::view::{
    ViewDepthTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms,
    prepare_view_targets,
};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy_auto_plugin::prelude::*;
use return_ok::{some_or_return, some_or_return_ok};

use crate::post_processing::PostProcessPlugin;

const SHADER_ASSET_PATH: &str = "post_processing_outlines.wgsl";

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = PostProcessPlugin)]
struct PostProcessOutlinesPlugin;

impl Plugin for PostProcessOutlinesPlugin {
    #[auto_plugin]
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PostProcessOutlinesSettings>::default(),
            UniformComponentPlugin::<PostProcessOutlinesSettings>::default(),
        ));

        let render_app = some_or_return!(app.get_sub_app_mut(RenderApp));

        render_app
            .add_systems(
                Render,
                (configure_view_targets, prepare_textures)
                    .after(prepare_view_targets)
                    .in_set(RenderSystems::ManageViews),
            )
            .add_render_graph_node::<ViewNodeRunner<PostProcessOutlinesNode>>(
                Core3d,
                PostProcessOutlinesLabel,
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = some_or_return!(app.get_sub_app_mut(RenderApp));

        render_app.init_resource::<PostProcessOutlinesPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct PostProcessOutlinesLabel;

#[derive(Default)]
struct PostProcessOutlinesNode;

impl ViewNode for PostProcessOutlinesNode {
    type ViewQuery = (
        &'static ViewUniformOffset,
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static PostProcessOutlinesSettings,
        &'static DynamicUniformIndex<PostProcessOutlinesSettings>,
        &'static PostProcessQuantizeBuffers,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            view_uniform_offset,
            view_target,
            view_depth,
            _post_process_settings,
            settings_index,
            blur_textures,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessOutlinesPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let depth_blit_pipeline = some_or_return_ok!(
            pipeline_cache.get_render_pipeline(post_process_pipeline.depth_blit_pipeline_id)
        );
        let blur_horizontal_pipeline = some_or_return_ok!(
            pipeline_cache.get_render_pipeline(post_process_pipeline.blur_horizontal_pipeline_id)
        );
        let blur_vertical_pipeline = some_or_return_ok!(
            pipeline_cache.get_render_pipeline(post_process_pipeline.blur_vertical_pipeline_id)
        );
        let main_pipeline = some_or_return_ok!(
            pipeline_cache.get_render_pipeline(post_process_pipeline.main_pipeline_id)
        );

        let view_uniforms = world.resource::<ViewUniforms>();
        let view_uniforms_binding = some_or_return_ok!(view_uniforms.uniforms.binding());

        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessOutlinesSettings>>();
        let settings_binding = some_or_return_ok!(settings_uniforms.uniforms().binding());

        let post_process = view_target.post_process_write();

        // Depth blit pass as the basis for the blur texture
        {
            let bind_group = render_context.render_device().create_bind_group(
                "depth_blit_post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    view_uniforms_binding.clone(),
                    post_process.source,
                    &post_process_pipeline.sampler,
                    view_depth.view(),
                    &blur_textures.horizontal_blur_texture.default_view, // ignore lol
                    settings_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("depth_blit_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &blur_textures.vertical_blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(depth_blit_pipeline);
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        // Blur horizontal pass
        {
            let bind_group = render_context.render_device().create_bind_group(
                "blur_horizontal_post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    view_uniforms_binding.clone(),
                    post_process.source,
                    &post_process_pipeline.sampler,
                    view_depth.view(),
                    &blur_textures.vertical_blur_texture.default_view, // ignore lol
                    settings_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("blur_horizontal_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &blur_textures.horizontal_blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(blur_horizontal_pipeline);
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        // Blur vertical pass
        {
            let bind_group = render_context.render_device().create_bind_group(
                "blur_vertical_post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    view_uniforms_binding.clone(),
                    post_process.source,
                    &post_process_pipeline.sampler,
                    view_depth.view(),
                    &blur_textures.horizontal_blur_texture.default_view, // the horizontal one we just did
                    settings_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("blur_vertical_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &blur_textures.vertical_blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(blur_vertical_pipeline);
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        // Main
        {
            let bind_group = render_context.render_device().create_bind_group(
                "main_post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    view_uniforms_binding.clone(),
                    post_process.source,
                    &post_process_pipeline.sampler,
                    view_depth.view(),
                    &blur_textures.vertical_blur_texture.default_view,
                    settings_binding.clone(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("main_post_process_pass"),
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

            render_pass.set_render_pipeline(main_pipeline);
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}

#[auto_resource(plugin = PostProcessOutlinesPlugin, derive)]
struct PostProcessOutlinesPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    depth_blit_pipeline_id: CachedRenderPipelineId,
    blur_horizontal_pipeline_id: CachedRenderPipelineId,
    blur_vertical_pipeline_id: CachedRenderPipelineId,
    main_pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessOutlinesPipeline {
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
                    texture_depth_2d(),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    uniform_buffer::<PostProcessOutlinesSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.load_asset(SHADER_ASSET_PATH);

        let fullscreen_shader = world.resource::<FullscreenShader>().clone();

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let depth_blit_pipeline_id =
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("depth_blit_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    entry_point: Some("depth_blit".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::R32Float,
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

        let blur_horizontal_pipeline_id =
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("blur_horizontal_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    entry_point: Some("blur_horizontal".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::R32Float,
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

        let blur_vertical_pipeline_id =
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("blur_vertical_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    entry_point: Some("blur_vertical".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::R32Float,
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

        let main_pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("main_post_process_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
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

        Self {
            layout,
            sampler,
            depth_blit_pipeline_id,
            blur_horizontal_pipeline_id,
            blur_vertical_pipeline_id,
            main_pipeline_id,
        }
    }
}

#[auto_component(plugin = PostProcessOutlinesPlugin, derive(Default, Clone, Copy, ExtractComponent, ShaderType), reflect, register)]
pub struct PostProcessOutlinesSettings {
    pub radius: f32,
}

/// Configures depth textures so that the depth of field shader can read from
/// them.
///
/// By default, the depth buffers that Bevy creates aren't able to be bound as
/// textures. The depth of field shader, however, needs to read from them. So we
/// need to set the appropriate flag to tell Bevy to make samplable depth
/// buffers.
fn configure_view_targets(
    mut view_targets: Query<&mut Camera3d, With<PostProcessOutlinesSettings>>,
) {
    for mut camera_3d in view_targets.iter_mut() {
        let mut depth_texture_usages = TextureUsages::from(camera_3d.depth_texture_usages);
        depth_texture_usages |= TextureUsages::TEXTURE_BINDING;
        camera_3d.depth_texture_usages = depth_texture_usages.into();
    }
}

#[auto_component(plugin = PostProcessOutlinesPlugin, derive)]
struct PostProcessQuantizeBuffers {
    horizontal_blur_texture: CachedTexture,
    vertical_blur_texture: CachedTexture,
}

fn prepare_textures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    mut view_targets: Query<(Entity, &ViewTarget), With<PostProcessOutlinesSettings>>,
) {
    for (entity, view_target) in view_targets.iter_mut() {
        let horizontal_blur_texture_descriptor = TextureDescriptor {
            label: Some("post_process_horizontal_blur_texture"),
            size: view_target.main_texture().size(),
            mip_level_count: 1,
            sample_count: view_target.main_texture().sample_count(),
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let horizontal_blur_texture =
            texture_cache.get(&render_device, horizontal_blur_texture_descriptor);

        let vertical_blur_texture_descriptor = TextureDescriptor {
            label: Some("post_process_vertical_blur_texture"),
            size: view_target.main_texture().size(),
            mip_level_count: 1,
            sample_count: view_target.main_texture().sample_count(),
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let vertical_blur_texture =
            texture_cache.get(&render_device, vertical_blur_texture_descriptor);

        commands.entity(entity).insert(PostProcessQuantizeBuffers {
            horizontal_blur_texture,
            vertical_blur_texture,
        });
    }
}
