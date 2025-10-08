use std::f32::consts::PI;
use std::time::Duration;

use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_butler::*;
use bevy_rapier3d::geometry::Collider;
use return_ok::{ok_or_continue, some_or_return_ok};

use crate::entity::spawner::{ActivateSpawner, Spawn, SpawnSystems, SpawnerActivatedSet};
use crate::entity::{
    EntityKilledSet, GelViscosity, Kill, Movement, RotateTowardMovement, SpawnHealthBar,
    TargetPlayer,
};
use crate::main_bundles::Mob;
use crate::npcs::NpcPlugin;
use crate::player_controller::weapons::Damage;
use crate::util::AnimationRootReference;

use super::name_tags::{NameTagAssets, SpawnNameTag};

#[derive(Component)]
pub struct Imp;

#[derive(Component)]
pub struct ImpSpawner;

#[derive(Component)]
pub struct InsertImpAssets;

#[derive(Resource)]
pub struct ImpAssets {
    pub model: Handle<Gltf>,
    pub ambient_sound_1: Handle<AudioSource>,
    pub ambient_sound_2: Handle<AudioSource>,
    pub hurt_sound: Handle<AudioSource>,
    pub death_sound: Handle<AudioSource>,

    pub sound_effect_variance: f32,

    pub ambient_sound_time: Duration,
    pub ambient_sound_time_variance: Duration,
}

impl ImpAssets {
    pub fn random_ambient_sound(&self) -> &Handle<AudioSource> {
        if rand::random::<f32>() < 0.5 {
            &self.ambient_sound_1
        } else {
            &self.ambient_sound_2
        }
    }

    pub fn random_sound_effect_variance(&self) -> f32 {
        rand::random::<f32>() * self.sound_effect_variance * 2.0 + 1.0 - self.sound_effect_variance
    }

    pub fn random_ambient_sound_time(&self) -> Duration {
        Duration::from_secs_f32(
            rand::random::<f32>() * self.ambient_sound_time_variance.as_secs_f32() * 2.0
                + self.ambient_sound_time.as_secs_f32()
                - self.ambient_sound_time_variance.as_secs_f32(),
        )
    }
}

#[derive(Component)]
pub struct ImpAnimations {
    pub idle: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub _attack: AnimationNodeIndex,
}

#[add_system(
	plugin = NpcPlugin, schedule = Startup,
)]
fn setup_imp_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ImpAssets {
        model: asset_server.load("imp.glb"),
        ambient_sound_1: asset_server.load("imp_ambient_1.ogg"),
        ambient_sound_2: asset_server.load("imp_ambient_2.ogg"),
        hurt_sound: asset_server.load("imp_hurt.ogg"),
        death_sound: asset_server.load("imp_death.ogg"),

        sound_effect_variance: 0.3,
        ambient_sound_time: Duration::from_secs_f32(5.0),
        ambient_sound_time_variance: Duration::from_secs_f32(2.0),
    });
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnerActivatedSet,
)]
fn queue_spawning_imp(
    mut activate_spawner: MessageReader<ActivateSpawner>,
    mut commands: Commands,
    spawners: Query<(), With<ImpSpawner>>,
) {
    for ev in activate_spawner.read() {
        if spawners.get(ev.spawner).is_err() {
            continue;
        }

        commands.entity(ev.entity).insert((
            Name::new("Imp"),
            Transform::from_translation(ev.position),
            InsertImpAssets,
        ));
    }
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = queue_spawning_imp,
	in_set = SpawnSystems,
)]
fn spawn_imp(
    imps: Query<Entity, With<InsertImpAssets>>,
    mut spawn: MessageWriter<Spawn>,
    mut commands: Commands,
    imp_assets: Res<ImpAssets>,
    gltfs: Res<Assets<Gltf>>,
) -> Result {
    let imp_gltf = some_or_return_ok!(gltfs.get(&imp_assets.model));

    for imp in imps.iter() {
        commands
            .entity(imp)
            .insert((
                SceneRoot(imp_gltf.scenes[0].clone()),
                Mob,
                SpawnHealthBar,
                TargetPlayer,
                RotateTowardMovement,
                Imp,
                SpawnNameTag,
                AmbientSoundTimer::default(),
            ))
            .remove::<InsertImpAssets>()
            .with_child((
                Transform::from_translation(Vec3::Y * 0.5),
                Collider::capsule_y(0.25, 0.25),
            ))
            .observe(
                |scene_instance_ready: On<SceneInstanceReady>,
                 mut commands: Commands,
                 imp_assets: Res<ImpAssets>,
                 gltfs: Res<Assets<Gltf>>,
                 mut animation_graphs: ResMut<Assets<AnimationGraph>>,
                 children: Query<&Children>,
                 material_names: Query<&GltfMaterialName>,
                 name_tag_assets: Res<NameTagAssets>|
                 -> Result {
                    let imp_gltf = gltfs
                        .get(&imp_assets.model)
                        .ok_or("Gltf should be loaded by now")?;

                    let (animation_graph, nodes) = AnimationGraph::from_clips([
                        imp_gltf.named_animations["Idle"].clone(),
                        imp_gltf.named_animations["Run"].clone(),
                        imp_gltf.named_animations["Attack"].clone(),
                    ]);
                    let animation_graph = animation_graphs.add(animation_graph);

                    let imp_animations = ImpAnimations {
                        idle: nodes[0],
                        run: nodes[1],
                        _attack: nodes[2],
                    };

                    let mut animation_player = AnimationPlayer::default();

                    let mut transitions = AnimationTransitions::new();
                    transitions.play(&mut animation_player, imp_animations.idle, Duration::ZERO);

                    let scene = *children
                        .get(scene_instance_ready.entity)
                        .unwrap()
                        .last()
                        .unwrap();
                    let armature = children.get(scene).unwrap()[0];

                    commands.entity(armature).insert((
                        Transform::from_rotation(Quat::from_rotation_y(PI)),
                        AnimationGraphHandle(animation_graph),
                        transitions,
                        animation_player,
                        imp_animations,
                    ));

                    for child in children
                        .iter_descendants(scene_instance_ready.entity)
                        .filter(|child| {
                            material_names
                                .get(*child)
                                .is_ok_and(|name| name.0 == "Candy")
                        })
                    {
                        commands
                            .entity(child)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(MeshMaterial3d(name_tag_assets.master_material.clone()));
                    }

                    commands
                        .entity(scene_instance_ready.entity)
                        .insert(AnimationRootReference(armature));

                    Ok(())
                },
            );

        spawn.write(Spawn { _entity: imp });
    }

    Ok(())
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnSystems,
)]
fn update_imp_animations(
    mut imps: Query<(&Movement, &AnimationRootReference), With<Imp>>,
    mut animations: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &ImpAnimations,
    )>,
) {
    for (movement, scene_root) in imps.iter_mut() {
        let (mut animation_player, mut transitions, animations) =
            animations.get_mut(scene_root.0).unwrap();

        if movement.0.length() > 0.0 {
            if transitions
                .get_main_animation()
                .map(|index| index != animations.run)
                .unwrap_or(true)
            {
                transitions
                    .play(
                        &mut animation_player,
                        animations.run,
                        Duration::from_secs_f32(0.5),
                    )
                    .repeat();
            }
        } else if transitions
            .get_main_animation()
            .map(|index| index != animations.idle)
            .unwrap_or(true)
        {
            transitions
                .play(
                    &mut animation_player,
                    animations.idle,
                    Duration::from_secs_f32(0.5),
                )
                .repeat();
        }
    }
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn imp_hurt_sound(
    mut damage: MessageReader<Damage>,
    mut imps: Query<(&GelViscosity, &GlobalTransform, &mut AmbientSoundTimer), With<Imp>>,
    mut commands: Commands,
    imp_assets: Res<ImpAssets>,
) {
    for ev in damage.read() {
        let (health, transform, mut sound_timer) = ok_or_continue!(imps.get_mut(ev.victim));

        if ev.damage + health.value < 0.0 {
            // dead
            continue;
        }

        commands.spawn((
            Transform::from_translation(transform.translation()),
            AudioPlayer(imp_assets.hurt_sound.clone()),
            PlaybackSettings::DESPAWN
                .with_speed(imp_assets.random_sound_effect_variance())
                .with_spatial(true),
        ));

        sound_timer.0 = imp_assets.random_ambient_sound_time();
    }
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn imp_kill_sound(
    mut kill: MessageReader<Kill>,
    mut imps: Query<(&GlobalTransform, &mut AmbientSoundTimer), With<Imp>>,
    mut commands: Commands,
    imp_assets: Res<ImpAssets>,
) {
    for ev in kill.read() {
        let (transform, mut sound_timer) = ok_or_continue!(imps.get_mut(ev.0));

        commands.spawn((
            Transform::from_translation(transform.translation()),
            AudioPlayer(imp_assets.death_sound.clone()),
            PlaybackSettings::DESPAWN
                .with_speed(imp_assets.random_sound_effect_variance())
                .with_spatial(true),
        ));

        sound_timer.0 = imp_assets.random_ambient_sound_time();
    }
}

#[derive(Component, Default)]
pub struct AmbientSoundTimer(pub Duration);

#[add_system(
	plugin = NpcPlugin, schedule = Update,
)]
fn imp_ambient_sound(
    mut imps: Query<(&GlobalTransform, &mut AmbientSoundTimer), With<Imp>>,
    mut commands: Commands,
    imp_assets: Res<ImpAssets>,
    time: Res<Time>,
) {
    for (transform, mut sound_timer) in imps.iter_mut() {
        sound_timer.0 = match sound_timer.0.checked_sub(time.delta()) {
            Some(time) => time,
            None => {
                commands.spawn((
                    Transform::from_translation(transform.translation()),
                    AudioPlayer(imp_assets.random_ambient_sound().clone()),
                    PlaybackSettings::DESPAWN
                        .with_speed(imp_assets.random_sound_effect_variance())
                        .with_spatial(true),
                ));

                imp_assets.random_ambient_sound_time()
            }
        }
    }
}
