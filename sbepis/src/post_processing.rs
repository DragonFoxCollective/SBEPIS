use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
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
    BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
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

use crate::prelude::*;

const SHADER_ASSET_PATH: &str = "post_processing.wgsl";

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    #[auto_plugin]
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // The settings will be a component that lives in the main world but will
            // be extracted to the render world every frame.
            // This makes it possible to control the effect from the main world.
            // This plugin will take care of extracting it automatically.
            // It's important to derive [`ExtractComponent`] on [`PostProcessingSettings`]
            // for this plugin to work correctly.
            ExtractComponentPlugin::<PostProcessSettings>::default(),
            // The settings will also be the data used in the shader.
            // This plugin will prepare the component for the GPU by creating a uniform buffer
            // and writing the data to that buffer every frame.
            UniformComponentPlugin::<PostProcessSettings>::default(),
        ));

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                (configure_view_targets, prepare_textures)
                    .after(prepare_view_targets)
                    .in_set(RenderSystems::ManageViews),
            )
            // Bevy's renderer uses a render graph which is a collection of nodes in a directed acyclic graph.
            // It currently runs on each view/camera and executes each node in the specified order.
            // It will make sure that any node that needs a dependency from another node
            // only runs when that dependency is done.
            //
            // Each node can execute arbitrary work, but it generally runs at least one render pass.
            // A node only has access to the render world, so if you need data from the main world
            // you need to extract it manually or with the plugin like above.
            // Add a [`Node`] to the [`RenderGraph`]
            // The Node needs to impl FromWorld
            //
            // The [`ViewNodeRunner`] is a special [`Node`] that will automatically run the node for each view
            // matching the [`ViewQuery`]
            .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(
                // Specify the label of the graph, in this case we want the graph for 3d
                Core3d,
                // It also needs the label of the node
                PostProcessLabel,
            )
            .add_render_graph_edges(
                Core3d,
                // Specify the node ordering.
                // This will automatically create all required node edges to enforce the given ordering.
                (
                    Node3d::Tonemapping,
                    PostProcessLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<PostProcessPipeline>();
    }
}

// /// A key that uniquely identifies post process pipelines.
// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
// pub struct PostProcessPipelineKey {
//     /// Whether we're doing Gaussian or bokeh blur.
//     pass: PostProcessPass,
//     /// Whether we're using HDR.
//     hdr: bool,
//     /// Whether the render target is multisampled.
//     multisample: bool,
// }

// /// Identifies a specific post process render pass.
// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
// enum PostProcessPass {
//     /// The first, horizontal, Gaussian blur pass.
//     GaussianHorizontal,
//     /// The second, vertical, Gaussian blur pass.
//     GaussianVertical,
//     /// The actual outline pass
//     Outlines,
// }

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

// The post process node used for the render graph
#[derive(Default)]
struct PostProcessNode;

// The ViewNode trait is required by the ViewNodeRunner
impl ViewNode for PostProcessNode {
    // The node needs a query to gather data from the ECS in order to do its rendering,
    // but it's not a normal system so we need to define it manually.
    //
    // This query will only run on the view entity
    type ViewQuery = (
        &'static ViewUniformOffset,
        &'static ViewTarget,
        &'static ViewDepthTexture,
        // This makes sure the node only runs on cameras with the PostProcessSettings component
        &'static PostProcessSettings,
        // As there could be multiple post processing components sent to the GPU (one per camera),
        // we need to get the index of the one that is associated with the current view.
        &'static DynamicUniformIndex<PostProcessSettings>,
        &'static PostProcessTextures,
    );

    // Runs the node logic
    // This is where you encode draw commands.
    //
    // This will run on every view on which the graph is running.
    // If you don't want your effect to run on every camera,
    // you'll need to make sure you have a marker component as part of [`ViewQuery`]
    // to identify which camera(s) should run the effect.
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
        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let post_process_pipeline = world.resource::<PostProcessPipeline>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache
        let Some(depth_blit_pipeline) =
            pipeline_cache.get_render_pipeline(post_process_pipeline.depth_blit_pipeline_id)
        else {
            return Ok(());
        };
        let Some(blur_horizontal_pipeline) =
            pipeline_cache.get_render_pipeline(post_process_pipeline.blur_horizontal_pipeline_id)
        else {
            return Ok(());
        };
        let Some(blur_vertical_pipeline) =
            pipeline_cache.get_render_pipeline(post_process_pipeline.blur_vertical_pipeline_id)
        else {
            return Ok(());
        };
        let Some(main_pipeline) =
            pipeline_cache.get_render_pipeline(post_process_pipeline.main_pipeline_id)
        else {
            return Ok(());
        };

        // Get the view uniforms (contains the camera matrices)
        let view_uniforms = world.resource::<ViewUniforms>();
        let Some(view_uniforms_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        // Get the settings uniform binding
        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_target.post_process_write();

        {
            // The bind_group gets created each frame.
            //
            // Normally, you would create a bind_group in the Queue set,
            // but this doesn't work with the post_process_write().
            // The reason it doesn't work is because each post_process_write will alternate the source/destination.
            // The only way to have the correct source/destination for the bind_group
            // is to make sure you get it during the node execution.
            let bind_group = render_context.render_device().create_bind_group(
                "depth_blit_post_process_bind_group",
                &post_process_pipeline.layout,
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                &BindGroupEntries::sequential((
                    // Set the view uniform binding
                    view_uniforms_binding.clone(),
                    // Make sure to use the source view
                    post_process.source,
                    // Use the sampler created for the pipeline
                    &post_process_pipeline.sampler,
                    // Use the depth texture view
                    view_depth.view(),
                    // ignore lol
                    &blur_textures.horizontal_blur_texture.default_view,
                    // Set the settings binding
                    settings_binding.clone(),
                )),
            );

            // Begin the render pass
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("depth_blit_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    // We need to specify the post process destination view here
                    // to make sure we write to the appropriate texture.
                    view: &blur_textures.vertical_blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
            // using the pipeline/bind_group created above
            render_pass.set_render_pipeline(depth_blit_pipeline);
            // By passing in the index of the post process settings on this view, we ensure
            // that in the event that multiple settings were sent to the GPU (as would be the
            // case with multiple cameras), we use the correct one.
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        {
            // The bind_group gets created each frame.
            //
            // Normally, you would create a bind_group in the Queue set,
            // but this doesn't work with the post_process_write().
            // The reason it doesn't work is because each post_process_write will alternate the source/destination.
            // The only way to have the correct source/destination for the bind_group
            // is to make sure you get it during the node execution.
            let bind_group = render_context.render_device().create_bind_group(
                "blur_horizontal_post_process_bind_group",
                &post_process_pipeline.layout,
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                &BindGroupEntries::sequential((
                    // Set the view uniform binding
                    view_uniforms_binding.clone(),
                    // Make sure to use the source view
                    post_process.source,
                    // Use the sampler created for the pipeline
                    &post_process_pipeline.sampler,
                    // Use the depth texture view
                    view_depth.view(),
                    // ignore lol
                    &blur_textures.vertical_blur_texture.default_view,
                    // Set the settings binding
                    settings_binding.clone(),
                )),
            );

            // Begin the render pass
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("blur_horizontal_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    // We need to specify the post process destination view here
                    // to make sure we write to the appropriate texture.
                    view: &blur_textures.horizontal_blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
            // using the pipeline/bind_group created above
            render_pass.set_render_pipeline(blur_horizontal_pipeline);
            // By passing in the index of the post process settings on this view, we ensure
            // that in the event that multiple settings were sent to the GPU (as would be the
            // case with multiple cameras), we use the correct one.
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        {
            // The bind_group gets created each frame.
            //
            // Normally, you would create a bind_group in the Queue set,
            // but this doesn't work with the post_process_write().
            // The reason it doesn't work is because each post_process_write will alternate the source/destination.
            // The only way to have the correct source/destination for the bind_group
            // is to make sure you get it during the node execution.
            let bind_group = render_context.render_device().create_bind_group(
                "blur_vertical_post_process_bind_group",
                &post_process_pipeline.layout,
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                &BindGroupEntries::sequential((
                    // Set the view uniform binding
                    view_uniforms_binding.clone(),
                    // Make sure to use the source view
                    post_process.source,
                    // Use the sampler created for the pipeline
                    &post_process_pipeline.sampler,
                    // Use the depth texture view
                    view_depth.view(),
                    // the horizontal one we just did
                    &blur_textures.horizontal_blur_texture.default_view,
                    // Set the settings binding
                    settings_binding.clone(),
                )),
            );

            // Begin the render pass
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("blur_vertical_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    // We need to specify the post process destination view here
                    // to make sure we write to the appropriate texture.
                    view: &blur_textures.vertical_blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
            // using the pipeline/bind_group created above
            render_pass.set_render_pipeline(blur_vertical_pipeline);
            // By passing in the index of the post process settings on this view, we ensure
            // that in the event that multiple settings were sent to the GPU (as would be the
            // case with multiple cameras), we use the correct one.
            render_pass.set_bind_group(
                0,
                &bind_group,
                &[view_uniform_offset.offset, settings_index.index()],
            );
            render_pass.draw(0..3, 0..1);
        }

        {
            // The bind_group gets created each frame.
            //
            // Normally, you would create a bind_group in the Queue set,
            // but this doesn't work with the post_process_write().
            // The reason it doesn't work is because each post_process_write will alternate the source/destination.
            // The only way to have the correct source/destination for the bind_group
            // is to make sure you get it during the node execution.
            let bind_group = render_context.render_device().create_bind_group(
                "main_post_process_bind_group",
                &post_process_pipeline.layout,
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                &BindGroupEntries::sequential((
                    // Set the view uniform binding
                    view_uniforms_binding.clone(),
                    // Make sure to use the source view
                    post_process.source,
                    // Use the sampler created for the pipeline
                    &post_process_pipeline.sampler,
                    // Use the depth texture view
                    view_depth.view(),
                    // we should be done with the blur stuff by now
                    &blur_textures.vertical_blur_texture.default_view,
                    // Set the settings binding
                    settings_binding.clone(),
                )),
            );

            // Begin the render pass
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("main_post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    // We need to specify the post process destination view here
                    // to make sure we write to the appropriate texture.
                    view: post_process.destination,
                    resolve_target: None,
                    ops: Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
            // using the pipeline/bind_group created above
            render_pass.set_render_pipeline(main_pipeline);
            // By passing in the index of the post process settings on this view, we ensure
            // that in the event that multiple settings were sent to the GPU (as would be the
            // case with multiple cameras), we use the correct one.
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

// This contains global data used by the render pipeline. This will be created once on startup.
#[auto_resource(plugin = PostProcessPlugin, derive)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    depth_blit_pipeline_id: CachedRenderPipelineId,
    blur_horizontal_pipeline_id: CachedRenderPipelineId,
    blur_vertical_pipeline_id: CachedRenderPipelineId,
    main_pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // We need to define the bind group layout used for our pipeline
        let layout = render_device.create_bind_group_layout(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                // The layout entries will only be visible in the fragment stage
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    // The screen texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // The sampler that will be used to sample the screen texture
                    sampler(SamplerBindingType::Filtering),
                    // The depth texture (multisampled because msaa is on)
                    texture_depth_2d(),
                    // The blur texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // The settings uniform that will control the effect
                    uniform_buffer::<PostProcessSettings>(true),
                ),
            ),
        );

        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Get the shader handle
        let shader = world.load_asset(SHADER_ASSET_PATH);

        let fullscreen_shader = world.resource::<FullscreenShader>().clone();

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let depth_blit_pipeline_id = pipeline_cache
            // This will add the pipeline to the cache and queue its creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("depth_blit_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: Some("depth_blit".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::R32Float,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all fields can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            });

        let blur_horizontal_pipeline_id = pipeline_cache
            // This will add the pipeline to the cache and queue its creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("blur_horizontal_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: Some("blur_horizontal".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::R32Float,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all fields can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            });

        let blur_vertical_pipeline_id = pipeline_cache
            // This will add the pipeline to the cache and queue its creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("blur_vertical_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: Some("blur_vertical".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::R32Float,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all fields can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            });

        let main_pipeline_id = pipeline_cache
            // This will add the pipeline to the cache and queue its creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("main_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: Some("main".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all fields can have a default value.
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

#[auto_component(plugin = PostProcessPlugin, derive(Default, Clone, Copy, ExtractComponent, ShaderType), reflect, register)]
pub struct PostProcessSettings {
    pub intensity: f32,
    pub radius: f32,
}

#[auto_system(plugin = PostProcessPlugin, schedule = Update)]
fn update_settings(mut settings: Query<&mut PostProcessSettings>, time: Res<Time>) {
    for mut setting in &mut settings {
        let mut intensity = ops::sin(time.elapsed_secs());
        // Make it loop periodically
        intensity = ops::sin(intensity);
        // Remap it to 0..1 because the intensity can't be negative
        intensity = intensity * 0.5 + 0.5;
        // Scale it to a more reasonable level
        intensity *= 5.0;

        // Set the intensity.
        // This will then be extracted to the render world and uploaded to the GPU automatically by the [`UniformComponentPlugin`]
        setting.intensity = intensity;
    }
}

/// Configures depth textures so that the depth of field shader can read from
/// them.
///
/// By default, the depth buffers that Bevy creates aren't able to be bound as
/// textures. The depth of field shader, however, needs to read from them. So we
/// need to set the appropriate flag to tell Bevy to make samplable depth
/// buffers.
pub fn configure_view_targets(mut view_targets: Query<&mut Camera3d, With<PostProcessSettings>>) {
    for mut camera_3d in view_targets.iter_mut() {
        let mut depth_texture_usages = TextureUsages::from(camera_3d.depth_texture_usages);
        depth_texture_usages |= TextureUsages::TEXTURE_BINDING;
        camera_3d.depth_texture_usages = depth_texture_usages.into();
    }
}

/// The extra texture used as the second render target for the blur.
#[auto_component(plugin = PostProcessPlugin, derive)]
pub struct PostProcessTextures {
    horizontal_blur_texture: CachedTexture,
    vertical_blur_texture: CachedTexture,
}

/// Creates the second render target texture that the blur needs.
pub fn prepare_textures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    mut view_targets: Query<(Entity, &ViewTarget), With<PostProcessSettings>>,
) {
    for (entity, view_target) in view_targets.iter_mut() {
        // The texture matches the main view target texture.
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

        // The texture matches the main view target texture.
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

        commands.entity(entity).insert(PostProcessTextures {
            horizontal_blur_texture,
            vertical_blur_texture,
        });
    }
}
