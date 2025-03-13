use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::input::button_just_pressed;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::stamina::Stamina;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::CoyoteTimeSettings;
use super::di::DirectionalInput;
use super::grounded::EffectiveGrounded;
use super::walk::PlayerWalkSettings;

#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = PlayerDashSettings {
	speed_addon: 12.0,
	time: Duration::from_secs_f32(0.3),
	cooldown: Duration::from_secs_f32(0.2),
	stamina_cost: 0.33,
})]
pub struct PlayerDashSettings {
	pub speed_addon: f32,
	pub time: Duration,
	pub cooldown: Duration,
	pub stamina_cost: f32,
}

#[derive(Resource)]
pub struct DashAssets {
	pub sound: Handle<AudioSource>,
}

#[system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(DashAssets {
		sound: asset_server.load("ultrakill dash sound.mp3"),
	});
}

#[derive(Component, Default)]
pub struct TryingToDash(Duration);

#[derive(Component)]
pub struct Dashing {
	pub duration: Duration,
	pub velocity: Vec3,
}

#[derive(Component, Default)]
pub struct DashCooldown(Duration);

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateDashing,
)]
fn add_trying_to_dash(players: Query<Entity, With<PlayerBody>>, mut commands: Commands) {
	for player in players.iter() {
		commands.entity(player).insert(TryingToDash::default());
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateDashing,
	before = add_trying_to_dash,
)]
fn update_trying_to_dash(
	mut players: Query<(Entity, &mut TryingToDash)>,
	time: Res<Time>,
	coyote_time_settings: Res<CoyoteTimeSettings>,
	mut commands: Commands,
) {
	for (player, mut trying_to_dash) in players.iter_mut() {
		trying_to_dash.0 += time.delta();
		println!("Trying to dash: {:.2?}", trying_to_dash.0.as_secs_f32());
		if trying_to_dash.0 >= coyote_time_settings.input_buffer_time {
			commands.entity(player).remove::<TryingToDash>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateDashing,
)]
fn update_dash_cooldown(
	mut players: Query<(Entity, &mut DashCooldown)>,
	time: Res<Time>,
	dash_settings: Res<PlayerDashSettings>,
	mut commands: Commands,
) {
	for (player, mut dash_cooldown) in players.iter_mut() {
		dash_cooldown.0 += time.delta();
		if dash_cooldown.0 >= dash_settings.cooldown {
			commands.entity(player).remove::<DashCooldown>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateDi,
	after = MovementControlSet::UpdateGrounded,
	after = update_dash_cooldown,
	in_set = MovementControlSet::UpdateDashing,
)]
fn add_dashing(
	mut players: Query<
		(Entity, &Velocity, &DirectionalInput, &mut Stamina),
		(
			With<EffectiveGrounded>,
			With<TryingToDash>,
			Without<Dashing>,
			Without<DashCooldown>,
		),
	>,
	dash_settings: Res<PlayerDashSettings>,
	mut commands: Commands,
	assets: Res<DashAssets>,
) {
	for (player, velocity, di, mut stamina) in players.iter_mut() {
		if stamina.current >= dash_settings.stamina_cost {
			println!("Dashing!");
			commands
				.entity(player)
				.insert(Dashing {
					duration: Duration::ZERO,
					velocity: di.world_space.normalize_or(di.forward)
						* (velocity.linvel.length() + dash_settings.speed_addon),
				})
				.remove::<TryingToDash>();

			stamina.current -= dash_settings.stamina_cost;

			commands.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN));
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateDashing,
	before = add_trying_to_dash,
)]
fn update_dashing(
	mut players: Query<(Entity, &mut Dashing, &mut Movement, &mut Velocity)>,
	time: Res<Time>,
	dash_settings: Res<PlayerDashSettings>,
	walk_settings: Res<PlayerWalkSettings>,
	mut commands: Commands,
) {
	for (player, mut dashing, mut movement, mut velocity) in players.iter_mut() {
		dashing.duration += time.delta();
		if dashing.duration >= dash_settings.time {
			velocity.linvel = dashing.velocity.normalize_or_zero()
				* (dashing.velocity.length() - dash_settings.speed_addon
					+ (walk_settings.sprint_speed - walk_settings.speed));
			movement.0 = velocity.linvel;
			commands
				.entity(player)
				.remove::<Dashing>()
				.insert(DashCooldown::default());
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateDashing,
	in_set = MovementControlSet::DoHorizontalMovement,
)]
fn update_dash_velocity(mut movement: Query<(&mut Movement, &mut Velocity, &Dashing)>) {
	for (mut movement, mut velocity, dashing) in movement.iter_mut() {
		velocity.linvel = dashing.velocity;
		movement.0 = dashing.velocity;
	}
}
