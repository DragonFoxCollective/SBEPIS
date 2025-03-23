use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::camera::PlayerCamera;
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::util::find_in_ancestors;

use super::PlayerBody;

#[derive(Component)]
pub struct Pitch(pub f32);

/// Probably in radians per pixel?
#[derive(Resource)]
#[resource(plugin = PlayerControllerPlugin, init = MouseSensitivity(0.002))]
pub struct MouseSensitivity(pub f32);

#[system(
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
) {
	let delta = input.single().axis_pair(&PlayerAction::Look);

	{
		let (mut camera_transform, mut camera_pitch, camera) = player_camera.single_mut();
		if !camera.is_active {
			return;
		}

		camera_pitch.0 += delta.y * sensitivity.0;
		camera_pitch.0 = camera_pitch.0.clamp(-PI / 2., PI / 2.);
		camera_transform.rotation = Quat::from_rotation_x(-camera_pitch.0);
	}

	{
		let (mut body_transform, mut body_velocity) = player_body.single_mut();

		body_transform.rotation *= Quat::from_rotation_y(-delta.x * sensitivity.0);

		body_velocity.angvel = body_velocity
			.angvel
			.reject_from(body_transform.rotation * Vec3::Z);
	}
}

pub fn interact_with<T: Component>(
	rapier_context: Query<&RapierContext>,
	player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
	entities: Query<Entity, With<T>>,
	parents: Query<&Parent>,
	input: Query<&ActionState<PlayerAction>>,
	mut ev_interact: EventWriter<InteractedWith<T>>,
) {
	if !match input.iter().find(|input| !input.disabled()) {
		Some(input) => input.just_pressed(&PlayerAction::Interact),
		None => false,
	} {
		return;
	}

	let player_camera = player_camera.get_single().expect("Player camera missing");
	let mut hit_entity: Option<(Option<Entity>, f32)> = None;
	rapier_context.single().intersections_with_ray(
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
		ev_interact.send(InteractedWith::new(entity));
	}
}

#[derive(Event)]
pub struct InteractedWith<T>(pub Entity, PhantomData<T>);
impl<T> InteractedWith<T> {
	pub fn new(entity: Entity) -> Self {
		Self(entity, PhantomData)
	}
}
#[derive(SystemSet)]
pub struct InteractedWithSet<T>(PhantomData<T>);
impl<T> Default for InteractedWithSet<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
impl<T> std::fmt::Debug for InteractedWithSet<T> {
	fn fmt(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
		Ok(())
	}
}
impl<T> Clone for InteractedWithSet<T> {
	fn clone(&self) -> Self {
		Self(PhantomData)
	}
}
impl<T> PartialEq for InteractedWithSet<T> {
	fn eq(&self, _: &Self) -> bool {
		true
	}
}
impl<T> Eq for InteractedWithSet<T> {}
impl<T> std::hash::Hash for InteractedWithSet<T> {
	fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}
