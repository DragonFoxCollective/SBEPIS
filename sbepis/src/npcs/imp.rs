use std::time::Duration;

use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::geometry::Collider;
use return_ok::ok_or_return;

use super::name_tags::SpawnNameTag;
use crate::entity::spawner::{ActivateSpawner, Spawn};
use crate::entity::{
    GelViscosity, Kill, Movement, RotateTowardMovement, SpawnHealthBar, TargetPlayer,
};
use crate::main_bundles::Mob;
use crate::npcs::NpcPlugin;
use crate::player::weapons::Damage;
use crate::util::AnimationRoot;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct Imp;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct ImpSpawner;

#[auto_resource(plugin = NpcPlugin, derive, reflect, register)]
pub struct ImpAssets {
    pub model: Handle<Scene>,

    pub ambient_sound_1: Handle<AudioSource>,
    pub ambient_sound_2: Handle<AudioSource>,
    pub hurt_sound: Handle<AudioSource>,
    pub death_sound: Handle<AudioSource>,
    pub sound_effect_variance: f32,
    pub ambient_sound_time: Duration,
    pub ambient_sound_time_variance: Duration,

    pub animation_graph: Handle<AnimationGraph>,
    pub idle_animation: AnimationNodeIndex,
    pub run_animation: AnimationNodeIndex,
    pub _attack_animation: AnimationNodeIndex,
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

#[auto_system(plugin = NpcPlugin, schedule = Startup)]
fn setup_imp_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (animation_graph, nodes) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("imp.glb")),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("imp.glb")),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("imp.glb")),
    ]);
    let animation_graph = animation_graphs.add(animation_graph);

    commands.insert_resource(ImpAssets {
        model: asset_server.load(GltfAssetLabel::Scene(0).from_asset("imp.glb")),

        ambient_sound_1: asset_server.load("unlicensed/imp_ambient_1.ogg"),
        ambient_sound_2: asset_server.load("unlicensed/imp_ambient_2.ogg"),
        hurt_sound: asset_server.load("unlicensed/imp_hurt.ogg"),
        death_sound: asset_server.load("unlicensed/imp_death.ogg"),
        sound_effect_variance: 0.3,
        ambient_sound_time: Duration::from_secs_f32(5.0),
        ambient_sound_time_variance: Duration::from_secs_f32(2.0),

        animation_graph,
        idle_animation: nodes[1],
        run_animation: nodes[2],
        _attack_animation: nodes[0],
    });
}

#[auto_observer(plugin = NpcPlugin)]
fn spawn_imp(
    spawn: On<ActivateSpawner>,
    mut commands: Commands,
    spawners: Query<(), With<ImpSpawner>>,
    imp_assets: Res<ImpAssets>,
) {
    if spawners.get(spawn.spawner).is_err() {
        return;
    }

    commands
        .entity(spawn.spawned_entity)
        .insert((
            Name::new("Imp"),
            Transform::from_translation(spawn.position),
            SceneRoot(imp_assets.model.clone()),
            Mob,
            SpawnHealthBar,
            TargetPlayer,
            RotateTowardMovement,
            Imp,
            SpawnNameTag,
            AmbientSoundTimer::default(),
        ))
        .with_child((
            Transform::from_translation(Vec3::Y * 0.5),
            Collider::capsule_y(0.25, 0.25),
        ))
        .observe(
            |scene_ready: On<SceneInstanceReady>,
             imp_assets: Res<ImpAssets>,
             mut commands: Commands,
             children: Query<&Children>,
             mut players: Query<(), With<AnimationPlayer>>| {
                for child in children.iter_descendants(scene_ready.entity) {
                    if players.get_mut(child).is_ok() {
                        commands.entity(child).insert((
                            AnimationGraphHandle(imp_assets.animation_graph.clone()),
                            AnimationTransitions::default(),
                        ));
                        commands
                            .entity(scene_ready.entity)
                            .insert(AnimationRoot(child));
                    }
                }
            },
        );

    commands.trigger(Spawn {
        entity: spawn.spawned_entity,
    });
}

#[auto_system(plugin = NpcPlugin, schedule = Update)]
fn update_imp_animations(
    mut imps: Query<(&Movement, &AnimationRoot), With<Imp>>,
    mut animations: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    imp_assets: Res<ImpAssets>,
) -> Result {
    for (movement, scene_root) in imps.iter_mut() {
        let (mut animation_player, mut transitions) = animations.get_mut(scene_root.0)?;

        if movement.0.length() > 0.0 {
            if transitions
                .get_main_animation()
                .map(|index| index != imp_assets.run_animation)
                .unwrap_or(true)
            {
                transitions
                    .play(
                        &mut animation_player,
                        imp_assets.run_animation,
                        Duration::from_secs_f32(0.5),
                    )
                    .repeat();
            }
        } else if transitions
            .get_main_animation()
            .map(|index| index != imp_assets.idle_animation)
            .unwrap_or(true)
        {
            transitions
                .play(
                    &mut animation_player,
                    imp_assets.idle_animation,
                    Duration::from_secs_f32(0.5),
                )
                .repeat();
        }
    }
    Ok(())
}

#[auto_observer(plugin = NpcPlugin)]
fn imp_hurt_sound(
    damage: On<Damage>,
    mut imps: Query<(&GelViscosity, &GlobalTransform, &mut AmbientSoundTimer), With<Imp>>,
    mut commands: Commands,
    imp_assets: Res<ImpAssets>,
) {
    let (health, transform, mut sound_timer) = ok_or_return!(imps.get_mut(damage.victim));

    if damage.damage + health.value < 0.0 {
        // dead
        return;
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

#[auto_observer(plugin = NpcPlugin)]
fn imp_kill_sound(
    kill: On<Kill>,
    mut imps: Query<(&GlobalTransform, &mut AmbientSoundTimer), With<Imp>>,
    mut commands: Commands,
    imp_assets: Res<ImpAssets>,
) {
    let (transform, mut sound_timer) = ok_or_return!(imps.get_mut(kill.victim));

    commands.spawn((
        Transform::from_translation(transform.translation()),
        AudioPlayer(imp_assets.death_sound.clone()),
        PlaybackSettings::DESPAWN
            .with_speed(imp_assets.random_sound_effect_variance())
            .with_spatial(true),
    ));

    sound_timer.0 = imp_assets.random_ambient_sound_time();
}

#[auto_component(plugin = NpcPlugin, derive(Default), reflect, register)]
pub struct AmbientSoundTimer(pub Duration);

#[auto_system(plugin = NpcPlugin, schedule = Update)]
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
