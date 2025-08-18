use crate::prelude::*;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy_butler::*;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
struct FrameratePlugin;

#[add_plugin(to_plugin = FrameratePlugin, init = FrameTimeDiagnosticsPlugin::new(1))]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

#[derive(Component)]
struct FpsText;

#[add_system(plugin = FrameratePlugin, schedule = Startup)]
fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((FpsText, Text::new("FPS: N/A")));
}

#[add_system(plugin = FrameratePlugin, schedule = Update)]
fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.0 = format!("FPS: {value:>4.0}");
        } else {
            // display "N/A" if we can't get a FPS measurement
            // add an extra space to preserve alignment
            text.0 = "FPS: N/A".into();
        }
    }
}
