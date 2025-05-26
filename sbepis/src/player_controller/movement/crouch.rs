use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::input::{button_is_pressed, button_is_released};
use crate::player_controller::movement::MovementControlSet;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::prelude::PlayerBody;

use super::charge::ChargeCrouching;
use super::slide::Sliding;
use super::stand::Standing;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin)]
pub struct CrouchingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_transform: Transform,
}

impl FromWorld for CrouchingAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        CrouchingAssets {
            mesh: Mesh3d(
                meshes.add(
                    Capsule3d::new(0.25, 0.5)
                        .mesh()
                        .rings(1)
                        .latitudes(8)
                        .longitudes(16)
                        .uv_profile(CapsuleUvProfile::Fixed),
                ),
            ),
            mesh_transform: Transform::from_translation(Vec3::Y * 0.5),
            collider: Collider::capsule_y(0.25, 0.25),
            collider_transform: Transform::from_translation(Vec3::Y * 0.5),
            camera_transform: Transform::from_translation(Vec3::Y * 0.75),
        }
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn to_crouching_assets(
    trigger: Trigger<OnAdd, (Crouching, Sliding, ChargeCrouching)>,
    players: Query<&PlayerBody>,
    assets: Res<CrouchingAssets>,
    mut commands: Commands,
) -> Result {
    let body = players.get(trigger.target())?;
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    commands.entity(body.camera).insert(assets.camera_transform);
    Ok(())
}

#[derive(Component, Default)]
pub struct Crouching;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_is_pressed(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn standing_to_crouching(players: Query<Entity, With<Standing>>, mut commands: Commands) {
    for player in players.iter() {
        commands
            .entity(player)
            .remove::<Standing>()
            .insert(Crouching);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_is_released(PlayerAction::Crouch),
	in_set = MovementControlSet::UpdateState,
)]
fn crouching_to_standing(players: Query<Entity, With<Crouching>>, mut commands: Commands) {
    for player in players.iter() {
        commands
            .entity(player)
            .remove::<Crouching>()
            .insert(Standing);
    }
}
