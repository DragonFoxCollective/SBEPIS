use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use return_ok::ok_or_return_ok;

use crate::camera::PlayerCamera;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::util::find_in_ancestors;

use super::PlayerBody;

#[derive(Component)]
pub struct Pitch(pub f32);

/// Probably in radians per pixel?
#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin, init = MouseSensitivity(0.002))]
pub struct MouseSensitivity(pub f32);

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = MovementControlSet::UpdateState,
)]
fn rotate_camera_and_body(
    input: Query<&ActionState<PlayerAction>>,
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
    let delta = ok_or_return_ok!(input.single()).axis_pair(&PlayerAction::Look);

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
    rapier_context: ReadRapierContext,
    player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
    entities: Query<Entity, With<T>>,
    parents: Query<&ChildOf>,
    input: Query<&ActionState<PlayerAction>>,
    mut commands: Commands,
) -> Result {
    if !match input.iter().find(|input| !input.disabled()) {
        Some(input) => input.just_pressed(&PlayerAction::Interact),
        None => false,
    } {
        return Ok(());
    }

    let player_camera = player_camera.single()?;
    let mut hit_entity: Option<(Option<Entity>, f32)> = None;
    rapier_context.single()?.intersect_ray(
        player_camera.translation(),
        player_camera.forward().into(),
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
