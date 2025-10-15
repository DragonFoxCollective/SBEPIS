use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed};
use bevy_rapier3d::prelude::*;

use crate::camera::PlayerCamera;
use crate::player_controller::{Interact, PlayerControllerPlugin};
use crate::util::find_in_ancestors;

use super::PlayerBody;

#[derive(Action)]
pub struct Look;

#[derive(Component)]
pub struct Pitch(pub f32);

/// Probably in radians per pixel?
#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = MouseSensitivity(0.002))]
pub struct MouseSensitivity(pub f32);

#[add_observer(plugin = PlayerControllerPlugin)]
fn rotate_camera_and_body(
    look: On<JustPressed<Look>>,
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
