use std::time::Duration;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::bundles::observe;
use bevy_pretty_nice_input::{Action, Condition, ConditionedBindingUpdate, JustPressed};
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::charge::{ChargingTime, PlayerChargeSettings};
use crate::player_controller::stamina::Stamina;
use crate::util::MapRangeBetween;

use super::charge::ChargeWalking;
use super::di::WalkDI;
use super::sprint::Sprinting;
use super::walk::{PlayerWalkSettings, Walking};

#[derive(Action)]
#[action(invalidate = false)]
pub struct Dash;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct PlayerDashSettings {
    pub speed_addon: f32,
    pub dash_time: Duration,
    pub stamina_cost: f32,

    pub charge_min_speed_addon: f32,
    pub charge_max_speed_addon: f32,
    pub charge_dash_time: Duration,
    pub charge_min_stamina_cost: f32,
    pub charge_max_stamina_cost: f32,
}

impl Default for PlayerDashSettings {
    fn default() -> Self {
        Self {
            speed_addon: 12.0,
            dash_time: Duration::from_secs_f32(0.3),
            stamina_cost: 0.33,

            charge_min_speed_addon: 12.0,
            charge_max_speed_addon: 40.0,
            charge_dash_time: Duration::from_secs_f32(0.3),
            charge_min_stamina_cost: 0.33,
            charge_max_stamina_cost: 0.66,
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
    mut players: Query<
        (&Velocity, &WalkDI, &mut Stamina, Option<&ChargingTime>),
        Or<(With<Walking>, With<Sprinting>, With<ChargeWalking>)>, // TODO: replace this with event input
    >,
    charge_settings: Res<PlayerChargeSettings>,
    settings: Res<PlayerDashSettings>,
    mut commands: Commands,
    assets: Res<DashAssets>,
) -> Result {
    let (velocity, di, mut stamina, charging) = players.get_mut(dash.input)?;

    let (speed_addon, dash_time, stamina_cost) = if let Some(charging) = charging {
        let (power, stamina_cost) = charging
            .power_and_stamina_cost_from_stamina(
                &charge_settings,
                stamina.current,
                settings.charge_min_stamina_cost,
                settings.charge_max_stamina_cost,
            )
            .ok_or(BevyError::from(
                "Don't have enough stamina to charge dash, despite being in dash transition",
            ))?;
        (
            power.map_from_01(settings.charge_min_speed_addon..settings.charge_max_speed_addon),
            settings.charge_dash_time,
            stamina_cost,
        )
    } else {
        (
            settings.speed_addon,
            settings.dash_time,
            settings.stamina_cost,
        )
    };

    stamina.current -= stamina_cost;

    commands
        .entity(dash.input)
        .insert(Dashing {
            duration: Duration::ZERO,
            max_duration: dash_time,
            velocity: di.world_space.normalize_or(di.forward)
                * (velocity.linvel.length() + speed_addon),
            speed_addon,
        })
        .remove::<AffectedByGravity>();

    commands.spawn((AudioPlayer(assets.sound.clone()), PlaybackSettings::DESPAWN));

    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::UpdateState,
	before = walking_to_dashing,
))]
fn update_dashing(
    mut players: Query<(Entity, &mut Dashing, &mut Movement, &mut Velocity)>,
    time: Res<Time>,
    walk_settings: Res<PlayerWalkSettings>,
    mut commands: Commands,
) {
    for (player, mut dashing, mut movement, mut velocity) in players.iter_mut() {
        dashing.duration += time.delta();
        if dashing.duration >= dashing.max_duration {
            velocity.linvel = dashing.velocity.normalize_or_zero()
                * (dashing.velocity.length() - dashing.speed_addon
                    + (walk_settings.sprint_speed - walk_settings.speed));
            movement.0 = velocity.linvel;

            commands
                .entity(player)
                .remove::<Dashing>()
                .insert(AffectedByGravity);
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
             players: Query<(&Stamina, Has<ChargingTime>)>,
             settings: Res<PlayerDashSettings>|
             -> Result {
                let (stamina, is_charging) = players.get(update.input)?;
                let required_stamina = if is_charging {
                    settings.charge_min_stamina_cost
                } else {
                    settings.stamina_cost
                };
                if update.data.is_zero() || stamina.current >= required_stamina {
                    commands.trigger(update.next());
                }
                Ok(())
            },
        )
    }
}
