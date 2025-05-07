use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::Velocity;

use crate::entity::{EntityKilledSet, EntityPlugin};
use crate::gravity::AffectedByGravity;
use crate::player_controller::weapons::EntityDamagedSet;
use crate::util::{Billboard, DespawnTimer};
use crate::{gridbox_material, gridbox_material_extra, util::MapRange};

#[derive(Component)]
pub struct GelViscosity {
    pub value: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct SpawnHealthBar;

#[derive(Component)]
pub struct GelVial {
    pub entity: Entity,
    pub health: f32,
    pub max_health: f32,
    pub root: Entity,
    pub glass: Entity,
    pub length: f32,
    pub height: f32,
}

#[add_system(
	plugin = EntityPlugin, schedule = Update,
)]
fn spawn_health_bars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    entities: Query<Entity, With<SpawnHealthBar>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).remove::<SpawnHealthBar>();

        let length = 1.0;
        let size = Vec3::new(length, 0.3, 0.3);
        let glass_outline = 0.01;
        let outline = 0.05;
        let height = 1.5;

        let root = commands.spawn((Name::new("Gel Vial Root"), Billboard)).id();

        commands.spawn((
            Name::new("Gel Vial Outline"),
            Transform::from_scale(Vec3::NEG_ONE),
            Mesh3d(meshes.add(Cuboid::from_size(size + Vec3::splat(outline)))),
            MeshMaterial3d(gridbox_material("grey1", &mut materials, &asset_server)),
            ChildOf(root),
        ));

        let glass = commands
            .spawn((
                Name::new("Gel Vial Glass"),
                Mesh3d(meshes.add(Cuboid::from_size(size + Vec3::splat(glass_outline)))),
                MeshMaterial3d(gridbox_material_extra(
                    "clear",
                    &mut materials,
                    &asset_server,
                    StandardMaterial {
                        alpha_mode: AlphaMode::Blend,
                        clearcoat: 1.0,
                        clearcoat_perceptual_roughness: 0.0,
                        ..default()
                    },
                )),
                ChildOf(root),
            ))
            .id();

        commands.spawn((
            Name::new("Gel Vial"),
            GelVial {
                entity,
                health: 0.,
                max_health: 5.,
                root,
                glass,
                length,
                height,
            },
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(gridbox_material("red", &mut materials, &asset_server)),
            ChildOf(root),
        ));
    }
}

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn despawn_invalid_health_bars(
    mut commands: Commands,
    health_bars: Query<&GelVial>,
    entities: Query<Entity>,
    transforms: Query<&Transform>,
) -> Result {
    for health_bar in health_bars.iter() {
        if entities.get(health_bar.entity).is_err() {
            let root_transform = transforms.get(health_bar.root)?;

            commands
                .entity(health_bar.glass)
                .remove::<ChildOf>()
                .insert((
                    Transform::from_translation(
                        root_transform.transform_point(Vec3::X * health_bar.length),
                    )
                    .with_rotation(root_transform.rotation),
                    AffectedByGravity::default(),
                    Velocity {
                        linvel: root_transform.right().as_vec3()
                            + root_transform.up().as_vec3() * 2.0,
                        angvel: root_transform.forward().as_vec3() * 90.0,
                    },
                    DespawnTimer::new(1.0),
                ));

            commands.entity(health_bar.root).despawn();
        }
    }
    Ok(())
}

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	after = spawn_health_bars,
	after = EntityDamagedSet,
)]
fn update_health_bars_health(mut health_bars: Query<&mut GelVial>, healths: Query<&GelViscosity>) {
    for mut health_bar in health_bars.iter_mut() {
        let Ok(health) = healths.get(health_bar.entity) else {
            continue;
        };
        health_bar.health = health.value;
        health_bar.max_health = health.max;
    }
}

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	after = update_health_bars_health,
)]
fn update_health_bars_size(
    mut health_bars: Query<(&GelVial, &mut Transform)>,
    mut transforms: Query<&mut Transform, Without<GelVial>>,
) {
    for (health_bar, mut transform) in health_bars.iter_mut() {
        let percentage = (health_bar.health / health_bar.max_health).max(0.0);
        transform.translation.x = percentage.map_range((health_bar.length * 0.5)..0.0);
        transform.scale = Vec3::new(percentage, 1.0, 1.0);

        let Ok([mut glass_transform, mut root_transform, entity_transform]) =
            transforms.get_many_mut([health_bar.glass, health_bar.root, health_bar.entity])
        else {
            continue;
        };

        glass_transform.translation.x = percentage.map_range(health_bar.length..0.0);

        root_transform.translation = entity_transform.transform_point(Vec3::Y * health_bar.height);
    }
}

#[derive(Component)]
pub struct Healing(pub f32);

#[add_system(
	plugin = EntityPlugin, schedule = Update,
	in_set = EntityDamagedSet,
)]
fn heal(mut healings: Query<(&Healing, &mut GelViscosity)>, time: Res<Time>) {
    for (healing, mut health) in healings.iter_mut() {
        health.value += healing.0 * time.delta_secs();
        health.value = health.value.min(health.max);
    }
}
