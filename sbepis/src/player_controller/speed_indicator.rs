use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::prelude::*;

#[butler_plugin]
pub struct SpeedIndicatorPlugin;

#[derive(Component)]
pub struct SpeedIndicator;

#[system(
	plugin = SpeedIndicatorPlugin, schedule = Startup,
)]
fn setup_speed_indicator(mut commands: Commands) {
	commands
		.spawn((
			PlayerCameraNode,
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
		))
		.with_child((SpeedIndicator, Text::new("Speed: None")));
}

#[system(
	plugin = SpeedIndicatorPlugin, schedule = Update,
)]
fn update_speed_indicator(
	mut indicator: Query<&mut Text, With<SpeedIndicator>>,
	player: Query<&Velocity, With<PlayerBody>>,
) {
	let player = player.single();
	let speed = player.linvel.length();
	let mut indicator = indicator.single_mut();
	indicator.0 = format!("Speed: {:.2}", speed);
}
