use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::prelude::ComponentBuffer;
use bevy_rapier3d::prelude::*;
use itertools::Itertools as _;
use return_ok::ok_or_return;

use crate::entity::Movement;
use crate::gravity::ComputedGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::charge::Charging;
use crate::player_controller::movement::crouch::Crouching;
use crate::player_controller::movement::dash::Dashing;
use crate::player_controller::movement::grounded::Grounded;
use crate::player_controller::movement::roll::Rolling;
use crate::player_controller::movement::slide::Sliding;
use crate::player_controller::movement::stand::Standing;
use crate::player_controller::movement::trip::{TripRecover, Tripping};
use crate::player_controller::movement::walk::Sprinting;
use crate::player_controller::movement::{MovementControlSystems, Moving};
use crate::prelude::*;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = PlayerControllerPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct MovementIndicatorsPlugin;

#[auto_component(plugin = MovementIndicatorsPlugin, derive, reflect, register)]
pub struct SpeedIndicator;

#[auto_system(plugin = MovementIndicatorsPlugin, schedule = OnEnter(GameState::InGame))]
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
            DespawnOnExit(GameState::InGame),
        ))
        .with_child((SpeedIndicator, Text::new("Speed: None")));
}

#[auto_system(plugin = MovementIndicatorsPlugin, schedule = Update)]
fn update_speed_indicator(
    mut indicator: Query<&mut Text, With<SpeedIndicator>>,
    player: Query<(&Transform, &Velocity, &ComputedGravity), With<PlayerBody>>,
) {
    let (transform, velocity, gravity) = ok_or_return!(player.single());
    let mut indicator = ok_or_return!(indicator.single_mut());
    let speed = velocity.linvel.length();
    let local_speed = (transform.rotation.inverse() * velocity.linvel)
        .xz()
        .length();
    let gravity = gravity.acceleration.length();
    indicator.0 =
        format!("Global speed: {speed:.2}\nLocal speed: {local_speed:.2}\nGravity: {gravity:.2}");
}

#[auto_component(plugin = MovementIndicatorsPlugin, derive, reflect, register)]
pub struct DebugState;

#[auto_system(plugin = MovementIndicatorsPlugin, schedule = OnEnter(GameState::InGame))]
fn setup_debug_state(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug State"),
        Text("State".to_owned()),
        TextLayout::new_with_justify(Justify::Right),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        DebugState,
        PlayerCameraNode,
        DespawnOnExit(GameState::InGame),
    ));
}

#[auto_system(plugin = MovementIndicatorsPlugin, schedule = Update, config(
	after = MovementControlSystems::DoHorizontalMovement,
	after = MovementControlSystems::DoVerticalMovement,
))]
fn check_states(
    players: Query<
        (
            Has<Standing>,
            Has<Moving>,
            Has<Sprinting>,
            Has<Crouching>,
            Has<Dashing>,
            Has<Charging>,
            (Has<Tripping>, Has<TripRecover>),
            Has<Sliding>,
            Has<Rolling>,
            Has<Grounded>,
            Has<ComponentBuffer<Grounded>>,
            Has<Movement>,
        ),
        With<PlayerBody>,
    >,
    mut debug_states: Query<&mut Text, With<DebugState>>,
) {
    let mut debug_state = ok_or_return!(debug_states.single_mut());
    for tup in players.iter() {
        let arr = [
            tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6.0, tup.6.1, tup.7, tup.8, tup.9,
            tup.10, tup.11,
        ];
        let has = arr
            .into_iter()
            .zip([
                ShortName::of::<Standing>(),
                ShortName::of::<Moving>(),
                ShortName::of::<Sprinting>(),
                ShortName::of::<Crouching>(),
                ShortName::of::<Dashing>(),
                ShortName::of::<Charging>(),
                ShortName::of::<Tripping>(),
                ShortName::of::<TripRecover>(),
                ShortName::of::<Sliding>(),
                ShortName::of::<Rolling>(),
                ShortName::of::<Grounded>(),
                ShortName::of::<ComponentBuffer<Grounded>>(),
                ShortName::of::<Movement>(),
            ])
            .filter_map(|(has, name)| if has { Some(name) } else { None })
            .join("\n");
        debug_state.0 = has;
    }
}

#[auto_system(plugin = MovementIndicatorsPlugin, schedule = Startup)]
fn gizmo_overlay(mut config_store: If<ResMut<GizmoConfigStore>>) {
    for (_, config, _) in config_store.iter_mut() {
        config.depth_bias = -1.0;
    }
}

#[auto_system(plugin = MovementIndicatorsPlugin, schedule = Update, config(
	after = MovementControlSystems::DoHorizontalMovement,
	after = MovementControlSystems::DoVerticalMovement,
))]
fn movement_direction_gizmos(
    mut gizmos: If<Gizmos>,
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
