use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, JustPressed, JustReleased};
use bevy_rapier3d::prelude::*;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::prelude::PlayerBody;

use super::crouch::Crouching;
use super::dash::Dashing;
use super::slide::Sliding;
use super::sneak::Sneaking;
use super::sprint::Sprinting;

#[derive(Action)]
pub struct RollCrouching;

#[derive(Action)]
pub struct RollSprinting;

#[derive(Resource)]
#[insert_resource(plugin = PlayerControllerPlugin)]
pub struct RollingAssets {
    pub mesh: Mesh3d,
    pub mesh_transform: Transform,
    pub collider: Collider,
    pub collider_transform: Transform,
    pub camera_transform: Transform,
}

impl FromWorld for RollingAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        RollingAssets {
            mesh: Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap())),
            mesh_transform: Transform::from_translation(Vec3::Y * 0.5),
            collider: Collider::ball(0.5),
            collider_transform: Transform::from_translation(Vec3::Y * 0.5),
            camera_transform: Transform::from_translation(Vec3::Y * 0.5),
        }
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn to_rolling_assets(
    add: On<Add, (Rolling,)>,
    players: Query<&PlayerBody>,
    assets: Res<RollingAssets>,
    mut commands: Commands,
) -> Result {
    let body = players.get(add.entity)?;
    commands
        .entity(body.mesh)
        .insert((assets.mesh.clone(), assets.mesh_transform));
    commands
        .entity(body.collider)
        .insert((assets.collider.clone(), assets.collider_transform));
    commands.entity(body.camera).insert(assets.camera_transform);
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn remove_movement(add: On<Add, Rolling>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .remove::<Movement>()
        .insert(AffectedByGravity);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn readd_movement(
    add: On<Add, Rolling>,
    velocities: Query<&Velocity>,
    mut commands: Commands,
) -> Result {
    let velocity = velocities.get(add.entity)?;
    commands
        .entity(add.entity)
        .insert(Movement(velocity.linvel));
    Ok(())
}

#[derive(Component)]
pub struct Rolling;

#[add_observer(plugin = PlayerControllerPlugin)]
fn sliding_or_sneaking_or_crouching_to_rolling(
    roll: On<JustPressed<RollCrouching>>,
    mut commands: Commands,
) {
    commands
        .entity(roll.input)
        .remove::<Sliding>()
        .remove::<Sneaking>()
        .remove::<Crouching>()
        .insert(Rolling);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn sprinting_to_rolling(roll: On<JustPressed<RollSprinting>>, mut commands: Commands) {
    commands
        .entity(roll.input)
        .remove::<Sprinting>()
        .remove::<Dashing>()
        .insert(Rolling);
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn rolling_to_sliding(roll: On<JustReleased<RollSprinting>>, mut commands: Commands) {
    commands
        .entity(roll.input)
        .remove::<Rolling>()
        .insert(Sliding::default());
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn rolling_to_sprinting_or_standing(roll: On<JustReleased<RollSprinting>>, mut commands: Commands) {
    commands
        .entity(roll.input)
        .remove::<Rolling>()
        .insert(Sprinting);
}
