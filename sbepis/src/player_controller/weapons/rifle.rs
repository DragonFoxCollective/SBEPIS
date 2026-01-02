use std::f32::consts::PI;

use bevy::animation::{AnimationEvent, AnimationTarget, AnimationTargetId, animated_field};
use bevy::ecs::entity::EntityHashSet;
use bevy::mesh::CapsuleUvProfile;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_pretty_nice_input::{Filter, binding1d, input};
use bevy_pretty_nice_menus::MenuInputOf;
use bevy_rapier3d::math::Real;
use bevy_rapier3d::plugin::ReadRapierContext;
use bevy_rapier3d::prelude::QueryFilter;

use crate::camera::PlayerCamera;
use crate::fray::FrayMusic;
use crate::gridbox_material;
use crate::player_controller::PlayerControllerPlugin;
use crate::player_controller::weapons::{
    ActiveWeapon, Attack, Hit, NextWeapon, PrevWeapon, WeaponAnimation, WeaponOf,
};
use crate::prelude::*;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct RiflePivot {
    barrel: Entity,
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct Rifle {
    pub damage: f32,
    pub wielder: Entity,
    pub allies: EntityHashSet,
    pub charge: u32,
    pub last_beat: u32,
    pub charge_rate: u32,
    pub max_charge: u32,
    pub full_charge_multiplier: f32,
    pub is_charging: bool,
    pub fire_sound: Handle<AudioSource>,
    pub charge_sound: Handle<AudioSource>,
}

impl Rifle {
    fn update_last_beat(&mut self, fray: &FrayMusic) {
        self.last_beat = fray.subbeats(self.charge_rate);
    }

    fn get_beat(&mut self, fray: &FrayMusic) -> u32 {
        fray.subbeats(self.charge_rate)
    }
}

pub fn spawn_rifle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    animations: &mut Assets<AnimationClip>,
    graphs: &mut Assets<AnimationGraph>,
    body: Entity,
) {
    let rifle_pivot_id = AnimationTargetId::from_iter(["Rifle Pivot"]);

    let blast_time = 0.5;
    let reload_time = blast_time + 1.25;

    let mut attack_animation = AnimationClip::default();
    attack_animation.add_curve_to_target(
        rifle_pivot_id,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(
                Quat::from_rotation_x(0.0),
                Quat::from_rotation_x(PI * 0.99),
                EaseFunction::QuarticOut,
            )
            .reparametrize_linear(Interval::new(0.0, blast_time).unwrap())
            .unwrap()
            .chain(
                EasingCurve::new(
                    Quat::from_rotation_x(PI * 0.99),
                    Quat::from_rotation_x(0.0),
                    EaseFunction::QuadraticIn,
                )
                .reparametrize_linear(Interval::new(blast_time, reload_time).unwrap())
                .unwrap(),
            )
            .unwrap(),
        ),
    );
    attack_animation.add_event(0.0, RifleFire);
    attack_animation.add_event(reload_time, RifleStartCharging);

    let (graph, animation_index) = AnimationGraph::from_clip(animations.add(attack_animation));

    let rifle_barrel = commands
        .spawn((
            Name::new("Rifle Barrel"),
            Transform::from_rotation(Quat::from_rotation_x(-PI / 2.)),
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
            Rifle {
                damage: 0.5,
                wielder: body,
                allies: EntityHashSet::from_iter(vec![body]),
                charge: 0,
                last_beat: 0,
                charge_rate: 2,
                max_charge: 4,
                full_charge_multiplier: 3.0,
                is_charging: false,
                fire_sound: asset_server.load("flute.wav"),
                charge_sound: asset_server.load("flute.wav"),
            },
            DespawnOnExit(GameState::InGame),
        ))
        .id();

    let rifle_pivot = commands
        .spawn((
            Name::new("Rifle Pivot"),
            Transform::from_translation(Vec3::new(0.25, 0.75, -0.5)),
            Visibility::default(),
            RiflePivot {
                barrel: rifle_barrel,
            },
            AnimationGraphHandle(graphs.add(graph)),
            AnimationPlayer::default(),
            WeaponAnimation(animation_index),
            ChildOf(body),
            DespawnOnExit(GameState::InGame),
            WeaponOf(body),
            (
                MenuInputOf(body),
                input!(
                    Attack,
                    Axis1D[binding1d::left_click()],
                    [Filter::<With<ActiveWeapon>>::default()]
                ),
                input!(
                    NextWeapon,
                    Axis1D[binding1d::scroll_up()],
                    [Filter::<With<ActiveWeapon>>::default()]
                ),
                input!(
                    PrevWeapon,
                    Axis1D[binding1d::scroll_down()],
                    [Filter::<With<ActiveWeapon>>::default()]
                ),
            ),
        ))
        .add_child(rifle_barrel)
        .id();
    commands.entity(rifle_pivot).insert(AnimationTarget {
        id: rifle_pivot_id,
        player: rifle_pivot,
    });
}

#[derive(AnimationEvent, Clone)]
struct RifleFire;

#[derive(AnimationEvent, Clone)]
struct RifleStartCharging;

#[auto_observer(plugin = PlayerControllerPlugin)]
fn on_rifle_fire(
    fire: On<RifleFire>,
    rifle_pivots: Query<&RiflePivot>,
    mut rifles: Query<&mut Rifle>,
    mut commands: Commands,
    rapier_context: ReadRapierContext,
    frays: Query<&FrayMusic>,
    player_cameras: Query<&GlobalTransform, With<PlayerCamera>>,
    parents: Query<&ChildOf>,
) -> Result {
    let rifle_pivot_entity = fire.trigger().animation_player;
    let rifle_pivot = rifle_pivots.get(rifle_pivot_entity)?;
    let rifle_barrel_entity = rifle_pivot.barrel;
    let mut rifle = rifles.get_mut(rifle_barrel_entity)?;

    let fray = frays.single()?;
    let rapier_context = rapier_context.single()?;
    let player_camera = player_cameras.single()?;

    commands.spawn((
        Name::new("Rifle Shot SFX"),
        AudioPlayer::new(rifle.fire_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

    rifle.is_charging = false;

    if let Some((hit_entity, _distance)) = rapier_context.cast_ray(
        player_camera.translation(),
        player_camera.forward().into(),
        Real::MAX,
        false,
        QueryFilter::new().predicate(&|entity| {
            for entity in std::iter::once(entity).chain(parents.iter_ancestors(entity)) {
                if rifle.allies.contains(&entity) {
                    return false;
                }
            }
            true
        }),
    ) {
        let charge_multiplier = if rifle.charge >= rifle.max_charge {
            rifle.full_charge_multiplier
        } else {
            1.0
        };
        rifle.charge = 0;
        let damage = fray.modify_fray_damage(rifle.damage) * charge_multiplier;
        let fray_modifier = fray.modify_fray_damage(1.0);
        commands.trigger(Hit {
            victim: hit_entity,
            perpetrator: rifle.wielder,
            allies: rifle.allies.clone(),
            damage,
            fray_modifier,
        });
    } else {
        debug!("No hit");
    }

    Ok(())
}

#[auto_observer(plugin = PlayerControllerPlugin)]
fn on_rifle_start_charging(
    charging: On<RifleStartCharging>,
    rifle_pivots: Query<&RiflePivot>,
    mut rifles: Query<&mut Rifle>,
    frays: Query<&FrayMusic>,
) -> Result {
    let rifle_pivot_entity = charging.trigger().animation_player;
    let rifle_pivot = rifle_pivots.get(rifle_pivot_entity)?;
    let rifle_barrel_entity = rifle_pivot.barrel;
    let mut rifle = rifles.get_mut(rifle_barrel_entity)?;

    let fray = frays.single()?;

    rifle.update_last_beat(fray);
    rifle.is_charging = true;

    Ok(())
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn charge_rifle(
    mut commands: Commands,
    mut rifle_barrels: Query<&mut Rifle>,
    fray: Query<&FrayMusic>,
) {
    for mut rifle_barrel in rifle_barrels.iter_mut() {
        if !rifle_barrel.is_charging {
            continue;
        }

        let beat = fray
            .single()
            .map(|fray| rifle_barrel.get_beat(fray))
            .unwrap_or_default();
        if rifle_barrel.charge < rifle_barrel.max_charge && rifle_barrel.last_beat != beat {
            rifle_barrel.charge += 1;

            commands.spawn((
                Name::new("Rifle Charge SFX"),
                AudioPlayer::new(rifle_barrel.charge_sound.clone()),
                PlaybackSettings::DESPAWN.with_speed(2.0),
            ));
        }
        if let Ok(fray) = fray.single() {
            rifle_barrel.update_last_beat(fray);
        }
    }
}
