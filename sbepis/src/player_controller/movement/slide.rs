use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::{Action, Updated};
use bevy_rapier3d::prelude::Velocity;
use return_ok::ok_or_return_ok;

use crate::entity::Movement;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::MovementControlSystems;
use crate::player_controller::movement::grounded::Grounded;
use crate::util::MapRange;

#[derive(Action)]
#[action(invalidate = false)]
pub struct Slide;

#[derive(Action)]
#[action(invalidate = false)]
pub struct SlideNeutral;

#[derive(Action)]
#[action(invalidate = false)]
pub struct SlideStand;

#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
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

impl Default for PlayerSlideSettings {
    fn default() -> Self {
        Self {
            speed_cap: 1.0,
            friction: 1.0,
            forward_friction: 0.0,
            brake_friction: 10.0,
            turn_factor: 2.0,
            turn_friction: 0.0,
            direction_physics_resistance: 0.9,
            speed_physics_resistance: 0.0,
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
pub struct Sliding {
    di: Vec2,
}

#[auto_component(plugin = PlayerControllerPlugin, derive(Default), reflect, register)]
pub struct NeutralSliding;

#[auto_component(plugin = PlayerControllerPlugin, derive(Debug), reflect, register)]
pub struct SlidingSound {
    entity: Entity,
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn update_di(di: On<Updated<SlideNeutral>>, mut players: Query<&mut Sliding>) -> Result {
    let mut sliding = ok_or_return_ok!(players.get_mut(di.input));
    sliding.di = di
        .data
        .as_2d()
        .ok_or::<BevyError>("SlideNeutral didn't have 2D data".into())?
        .clamp_length_max(1.0)
        * Vec2::new(1.0, -1.0);
    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_sliding_sound(
    mut slidings: Query<
        (Entity, Option<&SlidingSound>, Has<Grounded>),
        Or<(With<Sliding>, With<NeutralSliding>)>,
    >,
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
    slidings: Query<(Entity, &SlidingSound), (Without<Sliding>, Without<NeutralSliding>)>,
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
    mut players: Query<(&mut Movement, &Transform, &Velocity, &Sliding)>,
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

    for (mut movement, transform, velocity, sliding) in players.iter_mut() {
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
            let angle = sliding.di.angle_to(Vec2::Y).abs();
            easing
                .sample(angle)
                .map(|max_friction| {
                    sliding
                        .di
                        .length()
                        .map_range(slide_settings.friction..max_friction)
                })
                .unwrap_or_default()
        };
        let friction = sliding
            .di
            .length()
            .map_range(center_friction..outer_friction);
        let friction_speed = (current_speed - slide_settings.speed_cap).max(0.0);

        let friction = -time.delta_secs() * friction * friction_speed;
        let speed = current_speed + friction;

        let turn_angle = -slide_settings.turn_factor * sliding.di.x * time.delta_secs();
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
