use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_graph::RenderGraphExt;
use bevy_auto_plugin::prelude::*;
use return_ok::some_or_return;

use crate::post_processing::outlines::PostProcessOutlinesLabel;
use crate::post_processing::quantize::PostProcessQuantizeLabel;
use crate::prelude::*;

pub mod outlines;
pub mod quantize;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    #[auto_plugin]
    fn build(&self, app: &mut App) {
        let render_app = some_or_return!(app.get_sub_app_mut(RenderApp));

        render_app.add_render_graph_edges(
            Core3d,
            (
                Node3d::Tonemapping,
                PostProcessOutlinesLabel,
                PostProcessQuantizeLabel,
                Node3d::EndMainPassPostProcessing,
            ),
        );
    }
}
