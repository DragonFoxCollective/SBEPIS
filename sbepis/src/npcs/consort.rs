use std::time::Duration;

use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::geometry::Collider;

use super::name_tags::SpawnNameTag;
use crate::entity::spawner::{ActivateSpawner, Spawn};
use crate::entity::{Healing, Movement, RandomInput, RotateTowardMovement, SpawnHealthBar};
use crate::main_bundles::Mob;
use crate::npcs::NpcPlugin;
use crate::questing::{QuestGiver, SpawnQuestMarker};
use crate::util::AnimationRoot;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct Consort;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct ConsortSpawner;

#[auto_resource(plugin = NpcPlugin, derive, reflect, register)]
pub struct ConsortAssets {
    pub model: Handle<Scene>,

    pub animation_graph: Handle<AnimationGraph>,
    pub idle_animation: AnimationNodeIndex,
    pub run_animation: AnimationNodeIndex,
}

#[auto_system(plugin = NpcPlugin, schedule = Startup)]
fn setup_imp_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (animation_graph, nodes) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("consort.glb")),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("consort.glb")),
    ]);
    let animation_graph = animation_graphs.add(animation_graph);

    commands.insert_resource(ConsortAssets {
        model: asset_server.load(GltfAssetLabel::Scene(0).from_asset("consort.glb")),

        animation_graph,
        idle_animation: nodes[0],
        run_animation: nodes[1],
    });
}

#[auto_observer(plugin = NpcPlugin)]
fn spawn_consort(
    spawn: On<ActivateSpawner>,
    spawners: Query<(), With<ConsortSpawner>>,
    mut commands: Commands,
    consort_assets: Res<ConsortAssets>,
) {
    if spawners.get(spawn.spawner).is_err() {
        return;
    }

    commands
        .entity(spawn.spawned_entity)
        .insert((
            Name::new("Consort"),
            Transform::from_translation(spawn.position),
            SceneRoot(consort_assets.model.clone()),
            Mob,
            SpawnHealthBar,
            RandomInput::default(),
            Healing(0.2),
            RotateTowardMovement,
            Consort,
            QuestGiver::default(),
            SpawnQuestMarker,
            SpawnNameTag,
        ))
        .with_child((
            Transform::from_translation(Vec3::Y * 0.5),
            Collider::capsule_y(0.25, 0.25),
        ))
        .observe(
            |scene_ready: On<SceneInstanceReady>,
             consort_assets: Res<ConsortAssets>,
             mut commands: Commands,
             children: Query<&Children>,
             mut players: Query<(), With<AnimationPlayer>>| {
                for child in children.iter_descendants(scene_ready.entity) {
                    if players.get_mut(child).is_ok() {
                        commands.entity(child).insert((
                            AnimationGraphHandle(consort_assets.animation_graph.clone()),
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
    mut consorts: Query<(&Movement, &AnimationRoot), With<Consort>>,
    mut animations: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    consort_assets: Res<ConsortAssets>,
) -> Result {
    for (movement, scene_root) in consorts.iter_mut() {
        let (mut animation_player, mut transitions) = animations.get_mut(scene_root.0)?;

        if movement.0.length() > 0.0 {
            if transitions
                .get_main_animation()
                .map(|index| index != consort_assets.run_animation)
                .unwrap_or(true)
            {
                transitions
                    .play(
                        &mut animation_player,
                        consort_assets.run_animation,
                        Duration::from_secs_f32(0.5),
                    )
                    .repeat();
            }
        } else if transitions
            .get_main_animation()
            .map(|index| index != consort_assets.idle_animation)
            .unwrap_or(true)
        {
            transitions
                .play(
                    &mut animation_player,
                    consort_assets.idle_animation,
                    Duration::from_secs_f32(0.5),
                )
                .repeat();
        }
    }
    Ok(())
}
