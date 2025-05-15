use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::ActionState;

use crate::entity::Movement;
use crate::entity::movement::ExecuteMovementSet;
use crate::input::{button_just_pressed, button_just_released, button_pressed};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::crouch::Crouching;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;
use crate::util::MapRange;

use super::crouch::{CrouchingAssets, StandingAssets, to_crouching_assets, to_standing_assets};
use super::di::DirectionalInput;
use super::sneak::Sneaking;
use super::stand::Standing;
use super::walk::Walking;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerSlideSettings {
	speed_cap: 10.0,
	friction: 1.0,
	forward_friction: 0.1,
	brake_friction: 10.0,
	turn_factor: 0.2,
	turn_friction: 1.0,
	to_crouch_speed_threshold: 1.5,
})]
pub struct PlayerSlideSettings {
    pub speed_cap: f32,
    pub friction: f32,
    pub forward_friction: f32,
    pub brake_friction: f32,
    /// In (radians per second) / (meters per second)
    pub turn_factor: f32,
    pub turn_friction: f32,
    pub to_crouch_speed_threshold: f32,
}

#[derive(Resource)]
pub struct SlideAssets {
    pub sound: Handle<AudioSource>,
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SlideAssets {
        sound: asset_server.load("slide.mp3"),
    });
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct Sliding {
    pub current_friction: f32,
    pub sound: Entity,
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn walking_to_sliding(
    players: Query<(Entity, &PlayerBody), With<Walking>>,
    assets: Res<CrouchingAssets>,
    slide_assets: Res<SlideAssets>,
    mut commands: Commands,
) {
    for (player, body) in players.iter() {
        let sound = commands
            .spawn((
                AudioPlayer::new(slide_assets.sound.clone()),
                PlaybackSettings::LOOP,
            ))
            .id();
        commands.entity(player).remove::<Walking>().insert(Sliding {
            current_friction: 0.0,
            sound,
        });
        to_crouching_assets(body, &mut commands, &assets);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn sliding_to_standing_or_walking(
    players: Query<(Entity, &PlayerBody, &Sliding)>,
    assets: Res<StandingAssets>,
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
) -> Result {
    let input = input.single()?;
    for (player, body, sliding) in players.iter() {
        if button_pressed(input, &PlayerAction::Move) {
            commands.entity(player).remove::<Sliding>().insert(Walking);
        } else {
            commands.entity(player).remove::<Sliding>().insert(Standing);
        }
        commands.entity(sliding.sound).despawn();
        to_standing_assets(body, &mut commands, &assets);
    }
    Ok(())
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::UpdateState,
)]
fn sliding_to_crouching_or_sneaking(
    players: Query<(Entity, &Sliding, &Movement)>,
    mut commands: Commands,
    slide_settings: Res<PlayerSlideSettings>,
    input: Query<&ActionState<PlayerAction>>,
) -> Result {
    let input = input.single()?;
    for (player, sliding, movement) in players.iter() {
        if movement.0.length() > slide_settings.to_crouch_speed_threshold {
            continue;
        }

        if button_pressed(input, &PlayerAction::Move) {
            commands.entity(player).remove::<Sliding>().insert(Sneaking);
        } else {
            commands
                .entity(player)
                .remove::<Sliding>()
                .insert(Crouching);
        }
        commands.entity(sliding.sound).despawn();
    }
    Ok(())
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSet::DoHorizontalMovement,
	before = ExecuteMovementSet,
)]
fn update_slide_velocity(
    mut movement: Query<(&mut Movement, &Transform, &DirectionalInput), With<Sliding>>,
    slide_settings: Res<PlayerSlideSettings>,
    time: Res<Time>,
) -> Result {
    // This is stupid, why can't I store this anywhere?
    let easing = EasingCurve::new(
        slide_settings.brake_friction,
        slide_settings.turn_friction,
        EaseFunction::CircularInOut,
    )
    .reparametrize_linear(Interval::new(0.0, PI / 2.0).unwrap())
    .unwrap()
    .chain(
        EasingCurve::new(
            slide_settings.turn_friction,
            slide_settings.forward_friction,
            EaseFunction::CircularInOut,
        )
        .reparametrize_linear(Interval::new(PI / 2.0, PI).unwrap())
        .unwrap(),
    )
    .unwrap();

    for (mut movement, transform, di) in movement.iter_mut() {
        let velocity = (transform.rotation.inverse() * movement.0).xz();

        let friction = if velocity == Vec2::ZERO || di.input == Vec2::ZERO {
            slide_settings.friction
        } else {
            let angle = di.input.angle_to(Vec2::Y).abs();
            let max_friction = easing
                .sample(angle)
                .ok_or(format!("Angle out of bounds: {:?}", angle))?;
            di.input
                .length()
                .map_range(slide_settings.friction..max_friction)
        };

        let friction_velocity = di.input.y.min(0.0).abs().map_range(
            velocity
                ..((velocity.length() - slide_settings.speed_cap).max(0.0)
                    * velocity.normalize_or_zero()),
        );

        let friction = -time.delta_secs() * friction * friction_velocity;
        let velocity = velocity + friction;

        let turn_angle =
            slide_settings.turn_factor * velocity.length() * di.input.x * time.delta_secs();
        let velocity = Vec2::from_angle(turn_angle).rotate(velocity);

        movement.0 = transform.rotation * Vec3::new(velocity.x, 0.0, velocity.y);
    }

    Ok(())
}
