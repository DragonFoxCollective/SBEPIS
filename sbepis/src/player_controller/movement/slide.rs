use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::grounded::Grounded;
use crate::player_controller::movement::{MovementControlSystems, Moving, MovingOptExt as _};
use crate::util::MapRange;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect(Default), register, init)]
pub struct PlayerSlideSettings {
    pub speed_cap: f32,
    pub friction: f32,
    /// In (radians per second) / (meters per second)
    pub turn_factor: f32,
    pub direction_physics_resistance: f32,
    pub speed_physics_resistance: f32,
    pub slope_slip_angle: f32,
    #[reflect(ignore)]
    friction_easing: Box<dyn Curve<f32> + Send + Sync>,
}

impl Default for PlayerSlideSettings {
    fn default() -> Self {
        let brake_friction = 10.0;
        let turn_friction = 0.0;
        let forward_friction = 0.0;

        let easing = EasingCurve::new(brake_friction, turn_friction, EaseFunction::CircularInOut)
            .reparametrize_linear(Interval::new(0.0, PI / 2.0).unwrap())
            .unwrap()
            .chain(
                EasingCurve::new(turn_friction, forward_friction, EaseFunction::CircularInOut)
                    .reparametrize_linear(Interval::new(PI / 2.0, PI).unwrap())
                    .unwrap(),
            )
            .unwrap();

        Self {
            speed_cap: 1.0,
            friction: 1.0,
            turn_factor: 2.0,
            direction_physics_resistance: 0.9,
            speed_physics_resistance: 0.0,
            friction_easing: Box::new(easing),
            slope_slip_angle: 45.0f32.to_radians(),
        }
    }
}

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct SlideAssets {
    pub sound: Handle<AudioSource>,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Startup)]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SlideAssets {
        sound: asset_server.load("slide.mp3"),
    });
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct Sliding;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug), reflect, register)]
pub struct SlidingSound {
    entity: Entity,
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_sliding_sound(
    mut slidings: Query<(Entity, Option<&SlidingSound>, Has<Grounded>), With<Sliding>>,
    mut commands: Commands,
    slide_assets: Res<SlideAssets>,
) {
    for (entity, sound, grounded) in slidings.iter_mut() {
        if grounded && sound.is_none() {
            let sound = commands
                .spawn((
                    AudioPlayer::new(slide_assets.sound.clone()),
                    PlaybackSettings::LOOP,
                ))
                .id();
            commands
                .entity(entity)
                .insert(SlidingSound { entity: sound });
        } else if !grounded && let Some(sound) = sound {
            commands.entity(sound.entity).despawn();
            commands.entity(entity).remove::<SlidingSound>();
        }
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn remove_sliding_sound(
    slidings: Query<(Entity, &SlidingSound), Without<Sliding>>,
    mut commands: Commands,
) {
    for (entity, sound) in slidings {
        commands.entity(sound.entity).despawn();
        commands.entity(entity).remove::<SlidingSound>();
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update, config(
	in_set = MovementControlSystems::DoHorizontalMovement,
))]
fn update_slide_velocity(
    mut players: Query<(&mut Movement, &Transform, &Velocity, Option<&Moving>), With<Sliding>>,
    slide_settings: Res<PlayerSlideSettings>,
    time: Res<Time>,
) -> Result {
    for (mut movement, transform, velocity, moving) in players.iter_mut() {
        let di = moving.as_input();
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
            let angle = di.angle_to(Vec2::Y).abs();
            slide_settings
                .friction_easing
                .sample(angle)
                .map(|max_friction| di.length().map_range(slide_settings.friction..max_friction))
                .unwrap_or_default()
        };
        let friction = di.length().map_range(center_friction..outer_friction);
        let friction_speed = (current_speed - slide_settings.speed_cap).max(0.0);

        let friction = -time.delta_secs() * friction * friction_speed;
        let speed = current_speed + friction;

        let turn_angle = -slide_settings.turn_factor * di.x * time.delta_secs();
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
