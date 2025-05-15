use std::f32::consts::PI;

use bevy::animation::{AnimationTarget, AnimationTargetId, animated_field};
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;

use crate::fray::FrayMusic;
use crate::gridbox_material;
use crate::player_controller::weapons::{DamageSweep, EndDamageSweep, SweepPivot, WeaponAnimation};

#[derive(Component)]
struct HammerPivot {
    pub head: Entity,
}

#[derive(Component)]
struct Hammer {
    pub damage: f32,
    pub wielder: Entity,
    pub allies: EntityHashSet,
    pub woosh_sound: Handle<AudioSource>,
    pub smash_sound: Handle<AudioSource>,
}

pub fn spawn_hammer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    animations: &mut Assets<AnimationClip>,
    graphs: &mut Assets<AnimationGraph>,
    body: Entity,
) -> (Entity, Entity) {
    let hammer_pivot_id = AnimationTargetId::from_iter(["Hammer Pivot"]);

    let lead_in_time = 0.5;
    let follow_through_time = lead_in_time + 3.0;

    let mut attack_animation = AnimationClip::default();
    attack_animation.add_curve_to_target(
        hammer_pivot_id,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(
                Quat::from_rotation_x(0.0),
                Quat::from_rotation_x(-PI * 0.5),
                EaseFunction::ExponentialIn,
            )
            .reparametrize_linear(Interval::new(0.0, lead_in_time).unwrap())
            .unwrap()
            .chain(
                EasingCurve::new(
                    Quat::from_rotation_x(-PI * 0.5),
                    Quat::from_rotation_x(0.0),
                    EaseFunction::CubicInOut,
                )
                .reparametrize_linear(Interval::new(lead_in_time, follow_through_time).unwrap())
                .unwrap(),
            )
            .unwrap(),
        ),
    );
    attack_animation.add_event(0.0, HammerStart);
    attack_animation.add_event(lead_in_time, HammerSmash);

    let (graph, animation_index) = AnimationGraph::from_clip(animations.add(attack_animation));

    let hammer_head = commands
        .spawn((
            Name::new("Hammer Head"),
            Transform::default()
                .with_translation(Vec3::Y * 1.)
                .with_rotation(Quat::from_rotation_x(PI / 2.)),
            Mesh3d(
                meshes.add(
                    Capsule3d::new(0.1, 0.5)
                        .mesh()
                        .rings(1)
                        .latitudes(8)
                        .longitudes(16)
                        .uv_profile(CapsuleUvProfile::Fixed),
                ),
            ),
            MeshMaterial3d(gridbox_material("red", materials, asset_server)),
            Hammer {
                damage: 1.0,
                wielder: body,
                allies: EntityHashSet::from_iter(vec![body]),
                woosh_sound: asset_server.load("whoosh.mp3"),
                smash_sound: asset_server.load("concrete_break3.wav"),
            },
        ))
        .id();

    let hammer_pivot = commands
        .spawn((
            Name::new("Hammer Pivot"),
            Transform::from_translation(Vec3::Y * 0.75),
            HammerPivot { head: hammer_head },
            SweepPivot {
                sweeper_length: 0.2,
                sweep_depth: 0.5,
                sweep_height: 0.2,
            },
            AnimationGraphHandle(graphs.add(graph)),
            AnimationPlayer::default(),
            WeaponAnimation(animation_index),
            ChildOf(body),
        ))
        .add_child(hammer_head)
        .observe(on_hammer_start)
        .observe(on_hammer_smash)
        .id();
    commands.entity(hammer_pivot).insert(AnimationTarget {
        id: hammer_pivot_id,
        player: hammer_pivot,
    });

    (hammer_pivot, hammer_head)
}

#[derive(Event, Clone, Copy)]
struct HammerStart;

#[derive(Event, Clone, Copy)]
struct HammerSmash;

fn on_hammer_start(
    trigger: Trigger<HammerStart>,
    hammer_pivots: Query<&HammerPivot>,
    hammers: Query<(&Hammer, &GlobalTransform)>,
    mut commands: Commands,
) -> Result {
    let hammer_pivot_entity = trigger.target();
    let hammer_pivot = hammer_pivots.get(hammer_pivot_entity)?;
    let hammer_head_entity = hammer_pivot.head;
    let (hammer, transform) = hammers.get(hammer_head_entity)?;

    commands.entity(hammer_head_entity).insert(DamageSweep::new(
        *transform,
        hammer_pivot_entity,
        hammer.allies.clone(),
        hammer.wielder,
    ));

    commands.spawn((
        Name::new("Hammer Swing SFX"),
        AudioPlayer::new(hammer.woosh_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

    Ok(())
}

fn on_hammer_smash(
    trigger: Trigger<HammerSmash>,
    hammer_pivots: Query<&HammerPivot>,
    hammers: Query<&Hammer>,
    fray: Query<&FrayMusic>,
    mut commands: Commands,
) -> Result {
    let hammer_pivot_entity = trigger.target();
    let hammer_pivot = hammer_pivots.get(hammer_pivot_entity)?;
    let hammer_head_entity = hammer_pivot.head;
    let hammer = hammers.get(hammer_head_entity)?;

    let fray = fray.single()?;

    commands.entity(hammer_head_entity).insert(EndDamageSweep {
        damage: fray.modify_fray_damage(hammer.damage),
        fray_modifier: fray.modify_fray_damage(1.0),
    });

    commands.spawn((
        Name::new("Hammer Smash SFX"),
        AudioPlayer::new(hammer.smash_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

    Ok(())
}
