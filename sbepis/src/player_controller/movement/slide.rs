use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, JustReleased};
use bevy_rapier3d::prelude::Velocity;
use return_ok::some_or_return_ok;

use crate::entity::Movement;
use crate::entity::movement::ExecuteMovementSet;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::grounded::Grounded;
use crate::util::MapRange;

use super::di::DirectionalInput;
use super::grounded::GroundedContact;
use super::walk::Walking;

#[derive(Action)]
pub struct Slide;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = PlayerSlideSettings {
	speed_cap: 1.0,
	friction: 1.0,
	forward_friction: 0.0,
	brake_friction: 10.0,
	turn_factor: 2.0,
	turn_friction: 0.0,
	direction_physics_resistance: 0.9,
	speed_physics_resistance: 0.0,
})]
pub struct PlayerSlideSettings {
    pub speed_cap: f32,
    pub friction: f32,
    pub forward_friction: f32,
    pub brake_friction: f32,
    /// In (radians per second) / (meters per second)
    pub turn_factor: f32,
    pub turn_friction: f32,
    pub direction_physics_resistance: f32,
    pub speed_physics_resistance: f32,
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

#[derive(Component, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct Sliding {
    sound: Option<Entity>,
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_sliding_sound(
    mut slidings: Query<(&mut Sliding, Has<Grounded>)>,
    mut commands: Commands,
    slide_assets: Res<SlideAssets>,
) {
    for (mut sliding, grounded) in slidings.iter_mut() {
        if grounded && sliding.sound.is_none() {
            let sound = commands
                .spawn((
                    AudioPlayer::new(slide_assets.sound.clone()),
                    PlaybackSettings::LOOP,
                ))
                .id();
            sliding.sound = Some(sound);
        } else if !grounded && let Some(sound) = sliding.sound {
            commands.entity(sound).despawn();
            sliding.sound = None;
        }
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn remove_sliding_sound(
    remove: On<Remove, Sliding>,
    slidings: Query<&Sliding>,
    mut commands: Commands,
) -> Result {
    let sliding = slidings.get(remove.entity)?;
    commands.entity(some_or_return_ok!(sliding.sound)).despawn();
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn walking_to_sliding(slide: On<JustPressed<Slide>>, mut commands: Commands) {
    commands
        .entity(slide.input)
        .remove::<Walking>()
        .insert(Sliding::default());
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn sliding_to_walking(slide: On<JustReleased<Slide>>, mut commands: Commands) {
    commands
        .entity(slide.input)
        .remove::<Sliding>()
        .insert(Walking);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn readd_movement(remove: On<Remove, Sliding>, mut commands: Commands, players: Query<&Velocity>) {
    commands.entity(remove.entity).insert_if_new(
        players
            .get(remove.entity)
            .map(|velocity| Movement(velocity.linvel))
            .unwrap_or_default(),
    );
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = MovementControlSystems::DoHorizontalMovement,
	before = ExecuteMovementSet,
)]
fn update_slide_velocity(
    mut players: Query<
        (
            &mut Movement,
            &Transform,
            &DirectionalInput,
            &GroundedContact,
            &Velocity,
        ),
        With<Sliding>,
    >,
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

    for (mut movement, transform, di, _contact, velocity) in players.iter_mut() {
        let current_speed = slide_settings
            .speed_physics_resistance
            .map_range(velocity.linvel.length()..movement.0.length());
        let current_direction = slerp(
            velocity.linvel.normalize_or_zero(),
            movement.0.normalize_or_zero(),
            slide_settings.direction_physics_resistance,
        )
        .reject_from(transform.up().into())
            + velocity
                .linvel
                .normalize_or_zero()
                .project_onto(transform.up().into());

        let center_friction = slide_settings.friction;
        let outer_friction = {
            let angle = di.input.angle_to(Vec2::Y).abs();
            easing
                .sample(angle)
                .map(|max_friction| {
                    di.input
                        .length()
                        .map_range(slide_settings.friction..max_friction)
                })
                .unwrap_or_default()
        };
        let friction = di.input.length().map_range(center_friction..outer_friction);
        let friction_speed = (current_speed - slide_settings.speed_cap).max(0.0);

        let friction = -time.delta_secs() * friction * friction_speed;
        let speed = current_speed + friction;

        let turn_angle = -slide_settings.turn_factor * di.input.x * time.delta_secs();
        let direction =
            Quat::from_axis_angle(transform.up().into(), turn_angle) * current_direction;

        // let normal = contact.normal;
        // let binormal = direction.cross(normal);
        // let tangent = normal.cross(binormal);

        movement.0 = direction * speed;
    }

    Ok(())
}

fn slerp(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    if from == Vec3::ZERO {
        return to;
    }
    if to == Vec3::ZERO {
        return from;
    }
    let angle = from.angle_between(to);
    if angle < f32::EPSILON {
        return from;
    }
    ((1.0 - t) * angle).sin() / angle.sin() * from + (t * angle).sin() / angle.sin() * to
}
