use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::{Action, JustPressed, Pressed};
use bevy_rapier3d::prelude::*;

use crate::camera::PlayerCamera;
use crate::player_controller::{Interact, PlayerControllerPlugin};
use crate::util::find_in_ancestors;

use super::PlayerBody;

#[derive(Action)]
pub struct Look;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct Pitch(pub f32);

/// Probably in radians per mouse sensor pixel?
#[auto_resource(plugin = PlayerControllerPlugin, derive, reflect, register, init)]
pub struct MouseSensitivity(pub f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(0.0015)
    }
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn rotate_camera_and_body(
    look: On<Pressed<Look>>,
    sensitivity: Res<MouseSensitivity>,
    mut player_camera: Query<
        (&mut Transform, &mut Pitch, &Camera),
        (With<PlayerCamera>, Without<PlayerBody>),
    >,
    mut player_body: Query<
        (&mut Transform, &mut Velocity),
        (Without<PlayerCamera>, With<PlayerBody>),
    >,
) -> Result {
    let delta = look
        .data
        .as_2d()
        .ok_or::<BevyError>("Look action expects 2d data".into())?;

    {
        let (mut camera_transform, mut camera_pitch, camera) = player_camera.single_mut()?;
        if !camera.is_active {
            return Ok(());
        }

        camera_pitch.0 += delta.y * sensitivity.0;
        camera_pitch.0 = camera_pitch.0.clamp(-PI / 2., PI / 2.);
        camera_transform.rotation = Quat::from_rotation_x(-camera_pitch.0);
    }

    {
        let (mut body_transform, mut body_velocity) = player_body.single_mut()?;

        body_transform.rotation *= Quat::from_rotation_y(-delta.x * sensitivity.0);

        body_velocity.angvel = body_velocity
            .angvel
            .reject_from(body_transform.rotation * Vec3::Z);
    }

    Ok(())
}

pub fn interact_with<T: Component>(
    interact: On<JustPressed<Interact>>,
    bodies: Query<&PlayerBody>,
    rapier_context: ReadRapierContext,
    cameras: Query<&GlobalTransform, With<PlayerCamera>>,
    entities: Query<Entity, With<T>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) -> Result {
    let body = bodies.get(interact.input)?;
    let camera = cameras.get(body.camera)?;
    let mut hit_entity: Option<(Option<Entity>, f32)> = None;
    rapier_context.single()?.intersect_ray(
        camera.translation(),
        camera.forward().into(),
        3.0,
        true,
        QueryFilter::default(),
        |entity, intersection| {
            if hit_entity
                .map(|(_, time)| intersection.time_of_impact < time)
                .unwrap_or(true)
                && intersection.time_of_impact > 0.0
            {
                hit_entity = Some((
                    find_in_ancestors(entity, &entities, &parents),
                    intersection.time_of_impact,
                ));
            }
            true
        },
    );

    if let Some((Some(entity), _)) = hit_entity {
        commands.trigger(InteractWith::<T>::new(entity));
    }

    Ok(())
}

#[derive(EntityEvent)]
pub struct InteractWith<T> {
    pub entity: Entity,
    pub _marker: PhantomData<T>,
}
impl<T> InteractWith<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct PlayerFov(pub f32);

#[derive(Reflect)]
pub struct InterpolateFovCurve {
    pub fov: f32,
    pub duration_secs: f32,
    pub ease: EaseFunction,
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct InterpolateFov {
    pub curves: Vec<InterpolateFovCurve>,
}

impl InterpolateFov {
    pub fn new(fov: f32, duration_secs: f32) -> Self {
        Self {
            curves: vec![InterpolateFovCurve {
                fov,
                duration_secs,
                ease: EaseFunction::CircularOut,
            }],
        }
    }
}

type BoxedCurveInner = dyn Curve<f32> + Send + Sync;
type BoxedCurve = Box<BoxedCurveInner>;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
#[reflect(from_reflect = false)]
struct InterpolateFovBuilt {
    #[reflect(ignore)]
    easing: BoxedCurve,
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn build_interpolate_fov(
    add: On<Add, InterpolateFov>,
    players: Query<(&PlayerBody, &InterpolateFov)>,
    cameras: Query<&Projection>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
    let (player, fov) = players.get(add.entity)?;
    let projection = cameras.get(player.camera)?;
    let Projection::Perspective(projection) = projection else {
        return Ok(());
    };

    let mut easings = fov
        .curves
        .iter()
        .fold(
            (Vec::new(), projection.fov, time.elapsed_secs()),
            |(mut vec, old_fov, old_time), f| {
                let new_time = old_time + f.duration_secs;
                vec.push(
                    EasingCurve::new(old_fov, f.fov, f.ease)
                        .reparametrize_linear(Interval::new(old_time, new_time).unwrap())
                        .unwrap(),
                );
                (vec, f.fov, new_time)
            },
        )
        .0;
    let first: BoxedCurve = Box::new(easings.remove(0));
    // I am so bad at using boxes
    fn folder(a: BoxedCurve, b: LinearReparamCurve<f32, EasingCurve<f32>>) -> BoxedCurve {
        Box::new(a.chain(b).unwrap())
    }
    let easing: BoxedCurve = easings.into_iter().fold(first, folder);

    commands
        .entity(add.entity)
        .remove::<InterpolateFov>()
        .insert(InterpolateFovBuilt { easing });
    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn interpolate_fov(
    players: Query<(&PlayerBody, &InterpolateFovBuilt)>,
    mut cameras: Query<&mut Projection>,
    time: Res<Time>,
) -> Result {
    for (player, fov) in players.iter() {
        let mut projection = cameras.get_mut(player.camera)?;
        let Projection::Perspective(projection) = projection.as_mut() else {
            continue;
        };
        projection.fov = fov.easing.sample_clamped(time.elapsed_secs());
    }
    Ok(())
}
