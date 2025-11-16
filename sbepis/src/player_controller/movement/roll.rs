use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, Updated};
use bevy_rapier3d::prelude::*;
use return_ok::ok_or_return_ok;

use crate::entity::Movement;
use crate::gravity::AffectedByGravity;
use crate::player_controller::PlayerControllerPlugin;
use crate::prelude::PlayerBody;

#[derive(Action)]
pub struct CrouchRoll;

#[derive(Action)]
pub struct SprintRoll;

#[derive(Action)]
pub struct NeutralCrouchRoll;

#[derive(Action)]
pub struct RollNeutral;

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
fn update_di(di: On<Updated<RollNeutral>>, mut players: Query<&mut Rolling>) -> Result {
    let mut rolling = ok_or_return_ok!(players.get_mut(di.input));
    rolling.di = di
        .data
        .as_2d()
        .ok_or::<BevyError>("RollNeutral didn't have 2D data".into())?
        .clamp_length_max(1.0)
        * Vec2::new(1.0, -1.0);
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

#[derive(Component, Default)]
pub struct Rolling {
    di: Vec2,
}

#[derive(Component, Default)]
pub struct NeutralRolling;
