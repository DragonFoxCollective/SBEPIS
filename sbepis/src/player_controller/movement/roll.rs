use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::input::{button_just_pressed, button_just_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::movement::slide::sliding_to_crouching_or_sneaking;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::crouch::{Crouching, CrouchingAssets, to_crouching_assets};
use super::dash::Dashing;
use super::slide::{SlideAssets, Sliding};
use super::sneak::Sneaking;
use super::sprint::Sprinting;
use super::stand::{StandingAssets, to_standing_assets};

#[derive(Resource)]
pub struct RollingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_transform: Transform,
}

pub fn to_rolling_assets(body: &PlayerBody, commands: &mut Commands, assets: &RollingAssets) {
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    commands.entity(body.camera).insert(assets.camera_transform);
}

#[derive(Component)]
pub struct Rolling;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
	before = sliding_to_crouching_or_sneaking,
)]
fn sliding_or_sneaking_or_crouching_to_rolling(
    mut players: Query<(Entity, &PlayerBody), Or<(With<Sliding>, With<Sneaking>, With<Crouching>)>>,
    mut commands: Commands,
    assets: Res<RollingAssets>,
) {
    for (player, body) in players.iter_mut() {
        commands
            .entity(player)
            .remove::<Sliding>()
            .remove::<Sneaking>()
            .remove::<Crouching>()
            .remove::<Movement>()
            .insert(Rolling);
        to_rolling_assets(body, &mut commands, &assets);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn sprinting_to_rolling(
    mut players: Query<(Entity, &PlayerBody), With<Sprinting>>,
    mut commands: Commands,
    assets: Res<RollingAssets>,
) {
    for (player, body) in players.iter_mut() {
        commands
            .entity(player)
            .remove::<Sprinting>()
            .remove::<Dashing>()
            .remove::<Movement>()
            .insert(Rolling)
            .insert(AffectedByGravity);
        to_rolling_assets(body, &mut commands, &assets);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Sprint),
	in_set = MovementControlSet::UpdateState,
	after = rolling_to_sprinting,
)]
fn rolling_to_sliding(
    mut players: Query<(Entity, &PlayerBody), With<Rolling>>,
    mut commands: Commands,
    assets: Res<CrouchingAssets>,
    slide_assets: Res<SlideAssets>,
) {
    for (player, body) in players.iter_mut() {
        let sound = commands
            .spawn((
                AudioPlayer::new(slide_assets.sound.clone()),
                PlaybackSettings::LOOP,
            ))
            .id();

        commands
            .entity(player)
            .remove::<Rolling>()
            .insert(Sliding {
                current_friction: 0.0,
                sound,
            })
            .insert(Movement::default());

        to_crouching_assets(body, &mut commands, &assets);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn rolling_to_sprinting(
    mut players: Query<(Entity, &PlayerBody), With<Rolling>>,
    mut commands: Commands,
    assets: Res<StandingAssets>,
) {
    for (player, body) in players.iter_mut() {
        commands
            .entity(player)
            .remove::<Rolling>()
            .insert(Sprinting)
            .insert(Movement::default());
        to_standing_assets(body, &mut commands, &assets);
    }
}
