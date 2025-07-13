use std::any::type_name;

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use itertools::Itertools as _;

use crate::entity::Movement;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSet;
use crate::prelude::*;

use super::movement::charge::{ChargeCrouching, ChargeStanding, ChargeWalking};
use super::movement::crouch::Crouching;
use super::movement::dash::{Dashing, TryingToDash};
use super::movement::grounded::{EffectiveGrounded, Grounded};
use super::movement::jump::TryingToJump;
use super::movement::roll::Rolling;
use super::movement::slide::Sliding;
use super::movement::sneak::Sneaking;
use super::movement::sprint::Sprinting;
use super::movement::stand::Standing;
use super::movement::trip::{TripRecoverInAir, TripRecoverOnGround, Tripping, TryingToGroundParry};
use super::movement::walk::Walking;

#[butler_plugin]
#[add_plugin(to_plugin = PlayerControllerPlugin)]
#[derive(Default)]
pub struct MovementIndicatorsPlugin;

#[derive(Component)]
pub struct SpeedIndicator;

#[add_system(
	plugin = MovementIndicatorsPlugin, schedule = Startup,
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

#[add_system(
	plugin = MovementIndicatorsPlugin, schedule = Update,
)]
fn update_speed_indicator(
    mut indicator: Query<&mut Text, With<SpeedIndicator>>,
    player: Query<(&Transform, &Velocity), With<PlayerBody>>,
) -> Result {
    let (transform, velocity) = player.single()?;
    let speed = velocity.linvel.length();
    let local_speed = (transform.rotation.inverse() * velocity.linvel)
        .xz()
        .length();
    let mut indicator = indicator.single_mut()?;
    indicator.0 = format!(
        "Global speed: {:.2}\nLocal speed: {:.2}",
        speed, local_speed
    );
    Ok(())
}

#[derive(Component)]
pub struct DebugState;

#[add_system(
	plugin = MovementIndicatorsPlugin, schedule = Startup,
)]
fn setup_debug_state(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug State"),
        Text("State".to_owned()),
        TextLayout::new_with_justify(JustifyText::Right),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        DebugState,
        PlayerCameraNode,
    ));
}

#[add_system(
	plugin = MovementIndicatorsPlugin, schedule = Update,
	after = MovementControlSet::DoHorizontalMovement,
	after = MovementControlSet::DoVerticalMovement,
)]
fn check_states(
    players: Query<
        (
            Has<Standing>,
            Has<Walking>,
            Has<Sprinting>,
            Has<Crouching>,
            Has<Sneaking>,
            Has<Dashing>,
            (
                Has<ChargeStanding>,
                Has<ChargeCrouching>,
                Has<ChargeWalking>,
            ),
            (
                Has<Tripping>,
                Has<TripRecoverInAir>,
                Has<TripRecoverOnGround>,
            ),
            Has<Sliding>,
            Has<Rolling>,
            Has<Grounded>,
            Has<EffectiveGrounded>,
            (
                Has<TryingToDash>,
                Has<TryingToJump>,
                Has<TryingToGroundParry>,
            ),
            Has<Movement>,
        ),
        With<PlayerBody>,
    >,
    mut debug_states: Query<&mut Text, With<DebugState>>,
) -> Result {
    let mut debug_state = debug_states.single_mut()?;
    for tup in players.iter() {
        let arr = [
            tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6.0, tup.6.1, tup.6.2, tup.7.0, tup.7.1,
            tup.7.2, tup.8, tup.9, tup.10, tup.11, tup.12.0, tup.12.1, tup.12.2, tup.13,
        ];
        let has = arr
            .into_iter()
            .zip([
                type_name::<Standing>(),
                type_name::<Walking>(),
                type_name::<Sprinting>(),
                type_name::<Crouching>(),
                type_name::<Sneaking>(),
                type_name::<Dashing>(),
                type_name::<ChargeStanding>(),
                type_name::<ChargeCrouching>(),
                type_name::<ChargeWalking>(),
                type_name::<Tripping>(),
                type_name::<TripRecoverInAir>(),
                type_name::<TripRecoverOnGround>(),
                type_name::<Sliding>(),
                type_name::<Rolling>(),
                type_name::<Grounded>(),
                type_name::<EffectiveGrounded>(),
                type_name::<TryingToDash>(),
                type_name::<TryingToJump>(),
                type_name::<TryingToGroundParry>(),
                type_name::<Movement>(),
            ])
            .filter_map(|(has, name)| if has { Some(name) } else { None })
            .map(|name| name.split("::").last().unwrap())
            .join("\n");
        debug_state.0 = has;
    }
    Ok(())
}

#[add_system(
	plugin = MovementIndicatorsPlugin, schedule = Startup,
)]
fn gizmo_overlay(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, config, _) in config_store.iter_mut() {
        config.depth_bias = -1.0;
    }
}

#[add_system(
	plugin = MovementIndicatorsPlugin, schedule = Update,
	after = MovementControlSet::DoHorizontalMovement,
	after = MovementControlSet::DoVerticalMovement,
)]
fn movement_direction_gizmos(
    mut gizmos: Gizmos,
    players: Query<(&GlobalTransform, &Velocity, Option<&Movement>), With<PlayerBody>>,
) {
    for (transform, velocity, movement) in players.iter() {
        gizmos.ray(
            transform.translation(),
            velocity.linvel.normalize_or_zero(),
            css::RED,
        );
        gizmos.ray(
            transform.translation(),
            velocity
                .linvel
                .normalize_or_zero()
                .reject_from(transform.up().into()),
            css::PINK,
        );

        if let Some(movement) = movement {
            gizmos.ray(
                transform.translation(),
                movement.0.normalize_or_zero(),
                css::GREEN,
            );
            gizmos.ray(
                transform.translation(),
                movement
                    .0
                    .normalize_or_zero()
                    .reject_from(transform.up().into()),
                css::LIME,
            );
        }

        gizmos.ray_2d(Vec2::ZERO, Vec2::X, css::BLUE);
    }
}
