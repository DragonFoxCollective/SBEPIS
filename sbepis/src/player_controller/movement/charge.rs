use std::time::Instant;

use bevy::prelude::*;
use bevy_butler::*;

use crate::input::button_just_released;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::dash::add_trying_to_dash;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};

use super::dash::TryingToDash;
use super::grounded::EffectiveGrounded;
use super::stand::Standing;

#[derive(Resource)]
pub struct ChargeAssets {
	pub sound: Handle<AudioSource>,
}

#[system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(ChargeAssets {
		sound: asset_server.load("worms bazooka charge.mp3"),
	});
}

#[derive(Component)]
pub struct Charging {
	pub start_time: Instant,
	pub sound: Entity,
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateDi,
	after = MovementControlSet::UpdateGrounded,
	after = add_trying_to_dash,
	in_set = MovementControlSet::UpdateState,
)]
fn standing_to_charging(
	players: Query<
		Entity,
		(
			With<EffectiveGrounded>,
			With<TryingToDash>,
			With<Standing>,
			Without<Charging>,
		),
	>,
	mut commands: Commands,
	assets: Res<ChargeAssets>,
) {
	for player in players.iter() {
		println!("Charging!");

		let sound = commands
			.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::ONCE))
			.id();

		commands
			.entity(player)
			.insert(Charging {
				start_time: Instant::now(),
				sound,
			})
			.remove::<TryingToDash>()
			.remove::<Standing>();
	}
}

#[system(
    plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
)]
fn charging_to_standing(players: Query<(Entity, &Charging)>, mut commands: Commands) {
	for (player, charging) in players.iter() {
		commands.entity(charging.sound).despawn();
		commands
			.entity(player)
			.remove::<Charging>()
			.insert(Standing);
	}
}
