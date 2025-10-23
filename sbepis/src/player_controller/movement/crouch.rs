use bevy::mesh::CapsuleUvProfile;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, JustReleased};
use bevy_rapier3d::prelude::*;

use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::movement::sneak::Sneaking;
use crate::prelude::PlayerBody;

use super::charge::ChargeCrouching;
use super::slide::Sliding;
use super::stand::Standing;

#[derive(Action)]
pub struct Crouch;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin)]
pub struct CrouchingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_position: Vec3,
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
            camera_position: Vec3::Y * 0.75,
        }
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn to_crouching_assets(
    add: On<Add, (Crouching, Sliding, ChargeCrouching, Sneaking)>,
    players: Query<&PlayerBody>,
    mut cameras: Query<&mut Transform>,
    assets: Res<CrouchingAssets>,
    mut commands: Commands,
) -> Result {
    let body = players.get(add.entity)?;
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    cameras.get_mut(body.camera)?.translation = assets.camera_position;
    Ok(())
}

#[derive(Component, Default)]
pub struct Crouching;

#[add_observer(plugin = PlayerControllerPlugin)]
fn standing_to_crouching(crouch: On<JustPressed<Crouch>>, mut commands: Commands) {
    commands
        .entity(crouch.input)
        .remove::<Standing>()
        .insert(Crouching);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn crouching_to_standing(crouch: On<JustReleased<Crouch>>, mut commands: Commands) {
    commands
        .entity(crouch.input)
        .remove::<Crouching>()
        .insert(Standing);
}
