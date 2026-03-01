use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::bevy_event_chain::*;
use bevy_pretty_nice_input::bundles::observe;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::camera_controls::{InterpolateFov, PlayerFov};
use crate::player_controller::movement::charge::{Charging, PlayerChargeSettings};
use crate::player_controller::movement::walk::PlayerWalkSettings;
use crate::player_controller::movement::{MovementControlSystems, Moving, MovingOptExt as _};
use crate::player_controller::stamina::Stamina;
use crate::prelude::Player;
use crate::util::MapRange as _;

#[derive(Action)]
#[action(invalidate = false)]
pub struct Dash;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerDashSettings {
    pub speed_addon: f32,
    pub dash_time: Duration,
    pub stamina_cost: f32,

    pub charge_speed_addon: Range<f32>,
    pub charge_dash_time: Range<f32>,
    pub charge_stamina_cost: Range<f32>,

    pub fov_factor: f32,
    pub fov_ease_duration_secs: f32,
}

impl Default for PlayerDashSettings {
    fn default() -> Self {
        Self {
            speed_addon: 12.0,
            dash_time: Duration::from_secs_f32(0.3),
            stamina_cost: 0.33,

            charge_speed_addon: 12.0..40.0,
            charge_dash_time: 0.3..0.3,
            charge_stamina_cost: 0.33..0.66,

            fov_factor: 1.2,
            fov_ease_duration_secs: 0.3,
        }
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct DashAssets {
    pub sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DashAssets {
        sound: asset_server.load("ultrakill dash sound.mp3"),
    });
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct Dashing {
    pub duration: Duration,
    pub max_duration: Duration,
    pub velocity: Vec3,
    pub speed_addon: f32,
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn walking_to_dashing(
    dash: On<JustPressed<Dash>>,
    mut players: Query<(
        &Player,
        &Velocity,
        Option<&Moving>,
        &mut Stamina,
        Option<&Charging>,
        &PlayerFov,
    )>,
    cameras: Query<&GlobalTransform>,
    charge_settings: Res<PlayerChargeSettings>,
    settings: Res<PlayerDashSettings>,
    mut commands: Commands,
    assets: Res<DashAssets>,
) -> Result {
    let (player, velocity, moving, mut stamina, charging, fov) = players.get_mut(dash.input)?;

    let (speed_addon, dash_time, stamina_cost) = if let Some(charging) = charging {
        let power = charging
            .power_from_stamina(
                &charge_settings,
                stamina.current,
                settings.charge_stamina_cost.clone(),
            )
            .ok_or(BevyError::from(
                "Don't have enough stamina to charge dash, despite being in dash transition",
            ))?;
        (
            power.map_range(settings.charge_speed_addon.clone()),
            Duration::from_secs_f32(
                power.map_range(settings.charge_dash_time.start..settings.charge_dash_time.end),
            ),
            power.map_range(settings.charge_stamina_cost.clone()),
        )
    } else {
        (
            settings.speed_addon,
            settings.dash_time,
            settings.stamina_cost,
        )
    };

    stamina.current -= stamina_cost;

    let camera_transform = cameras.get(player.camera)?;
    let input = moving.as_input();
    let direction =
        camera_transform.rotation() * Vec3::new(input.x, 0.0, input.y).normalize_or(Vec3::NEG_Z);

    commands
        .entity(dash.input)
        .insert(Dashing {
            duration: Duration::ZERO,
            max_duration: dash_time,
            velocity: direction * (velocity.linvel.length() + speed_addon),
            speed_addon,
        })
        .insert(InterpolateFov::new(
            fov.0 * settings.fov_factor,
            settings.fov_ease_duration_secs,
        ))
        .remove::<AffectedByGravity>();

    commands.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN));

    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateState,
))]
fn update_dashing(
    mut players: Query<(
        Entity,
        &mut Dashing,
        &mut Movement,
        &mut Velocity,
        &PlayerFov,
    )>,
    time: Res<Time>,
    walk_settings: Res<PlayerWalkSettings>,
    settings: Res<PlayerDashSettings>,
    mut commands: Commands,
) {
    for (player, mut dashing, mut movement, mut velocity, fov) in players.iter_mut() {
        dashing.duration += time.delta();
        if dashing.duration >= dashing.max_duration {
            velocity.linvel = dashing.velocity.normalize_or_zero()
                * (dashing.velocity.length() - dashing.speed_addon
                    + (walk_settings.sprint_speed - walk_settings.speed));
            movement.0 = velocity.linvel;

            commands
                .entity(player)
                .remove::<Dashing>()
                .insert(AffectedByGravity)
                .insert(InterpolateFov::new(fov.0, settings.fov_ease_duration_secs));
        }
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::DoHorizontalMovement,
))]
fn update_dash_velocity(mut movement: Query<(&mut Movement, &mut Velocity, &Dashing)>) {
    for (mut movement, mut velocity, dashing) in movement.iter_mut() {
        velocity.linvel = dashing.velocity;
        movement.0 = dashing.velocity;
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct HasEnoughStaminaToDash;

impl Condition for HasEnoughStaminaToDash {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             players: Query<(&Stamina, Has<Charging>)>,
             settings: Res<PlayerDashSettings>|
             -> Result {
                let (stamina, is_charging) = players.get(update.input)?;
                let required_stamina = if is_charging {
                    settings.charge_stamina_cost.start
                } else {
                    settings.stamina_cost
                };
                if update.data.is_zero() || stamina.current >= required_stamina {
                    update.trigger_next(&mut commands);
                }
                Ok(())
            },
        )
    }
}
