use bevy::color::palettes::css;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Action, InputDisabled, JustPressed};
use bevy_rapier3d::na::Vector3;
use bevy_rapier3d::parry::shape::{self, SharedShape};
use bevy_rapier3d::prelude::*;
use return_ok::ok_or_return;

use crate::entity::{GelViscosity, Kill};
use crate::fray::FrayMusic;
use crate::player_controller::PlayerControllerPlugin;
use crate::util::{QuaternionEx, find_in_ancestors};

pub mod hammer;
pub mod rifle;
pub mod sword;

#[derive(EntityEvent)]
pub struct Hit {
    #[event_target]
    pub victim: Entity,
    pub perpetrator: Entity,
    pub allies: EntityHashSet,
    pub damage: f32,
    pub fray_modifier: f32,
}

#[derive(EntityEvent)]
pub struct Damage {
    #[event_target]
    pub victim: Entity,
    pub damage: f32,
    pub fray_modifier: f32,
}

#[derive(Component)]
pub struct DamageNumbers;

#[derive(Component)]
#[relationship_target(relationship = WeaponOf)]
pub struct Weapons {
    #[relationship]
    weapons: Vec<Entity>,
    pub active_weapon: Option<Entity>,
}

#[derive(Component)]
#[relationship(relationship_target = Weapons)]
pub struct WeaponOf(pub Entity);

#[derive(Component)]
pub struct UninitializedWeaponSet;

#[derive(Component)]
pub struct ActiveWeapon;

#[derive(Component)]
pub struct DamageSweep {
    pub hit_entities: EntityHashSet,
    pub last_transform: GlobalTransform,
    pub pivot: Entity,
    pub allies: EntityHashSet,
    pub owner: Entity,
}

#[derive(Component)]
pub struct EndDamageSweep {
    pub damage: f32,
    pub fray_modifier: f32,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct SweepPivot {
    pub sweeper_length: f32,
    pub sweep_depth: f32,
    pub sweep_height: f32,
}

impl DamageSweep {
    pub fn new(
        transform: GlobalTransform,
        pivot: Entity,
        allies: EntityHashSet,
        owner: Entity,
    ) -> Self {
        Self {
            hit_entities: EntityHashSet::default(),
            last_transform: transform,
            pivot,
            allies,
            owner,
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DebugColliderVisualizer;

#[derive(Component)]
pub struct WeaponAnimation(pub AnimationNodeIndex);

#[add_observer(plugin = PlayerControllerPlugin)]
fn attack(
    attack: On<JustPressed<Attack>>,
    mut weapons: Query<(&WeaponAnimation, &mut AnimationPlayer)>,
) -> Result {
    let (animation, mut animation_player) = weapons.get_mut(attack.input)?;

    if let Some(animation) = animation_player.animation_mut(animation.0) {
        if animation.is_finished() {
            animation.replay();
        }
    } else {
        animation_player.stop_all();
        animation_player.play(animation.0);
    }

    Ok(())
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn correct_animation_speed(
    fray_music: Query<&FrayMusic>,
    mut weapons: Query<(&WeaponAnimation, &mut AnimationPlayer)>,
) {
    let fray_music = ok_or_return!(fray_music.single());
    for (animation, mut animation_player) in weapons.iter_mut() {
        if let Some(animation) = animation_player.animation_mut(animation.0) {
            animation.set_speed(fray_music.speed());
        }
    }
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn sweep_dealers(
    mut commands: Commands,
    mut dealers: Query<(
        Entity,
        &mut DamageSweep,
        Option<&EndDamageSweep>,
        &GlobalTransform,
    )>,
    pivots: Query<(&SweepPivot, &GlobalTransform), Without<DamageSweep>>,
    rapier_context: ReadRapierContext,
    debug_collider_visualizers: Query<Entity, With<DebugColliderVisualizer>>,
) -> Result {
    let rapier_context = rapier_context.single()?;
    for (dealer_entity, mut dealer, end, transform) in dealers.iter_mut() {
        let (pivot, pivot_transform) = pivots.get(dealer.pivot)?;

        let start_tip = dealer
            .last_transform
            .transform_point(pivot.sweeper_length * 0.5 * Vec3::Z);
        let end_tip = transform.transform_point(pivot.sweeper_length * 0.5 * Vec3::NEG_Z);
        let delta = end_tip - start_tip;
        let position = (end_tip + start_tip) * 0.5;

        let pivot_position = pivot_transform.translation();
        let up = (end_tip - pivot_position).cross(start_tip - pivot_position);
        let rotation = Quat::from_look_to(delta, up);

        let sweep_shape = shape::Cuboid::new(Vector3::new(
            pivot.sweep_depth * 0.5,
            pivot.sweep_height * 0.5,
            delta.length() * 0.5,
        ));

        rapier_context.intersect_shape(
            position,
            rotation,
            &sweep_shape,
            QueryFilter::new(),
            |hit_entity| {
                dealer.hit_entities.insert(hit_entity);
                true
            },
        );
        if let Ok(debug_collider_visualizer) = debug_collider_visualizers.single() {
            commands
                .entity(debug_collider_visualizer)
                .insert(Collider::from(SharedShape::new(sweep_shape)))
                .insert(Transform::from_translation(position).with_rotation(rotation));
        }

        dealer.last_transform = *transform;

        if let Some(end) = end {
            for entity in dealer.hit_entities.iter() {
                commands.trigger(Hit {
                    victim: *entity,
                    perpetrator: dealer.owner,
                    allies: dealer.allies.clone(),
                    damage: end.damage,
                    fray_modifier: end.fray_modifier,
                });
            }

            commands
                .entity(dealer_entity)
                .remove::<DamageSweep>()
                .remove::<EndDamageSweep>();
        }
    }

    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn hit_to_damage(
    hit: On<Hit>,
    parents: Query<&ChildOf>,
    healths: Query<Entity, With<GelViscosity>>,
    mut commands: Commands,
) {
    let victim = find_in_ancestors(hit.victim, &healths, &parents).unwrap_or(hit.victim);
    if !hit.allies.contains(&victim) {
        commands.trigger(Damage {
            victim,
            damage: hit.damage,
            fray_modifier: hit.fray_modifier,
        });
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn deal_all_damage(
    damage: On<Damage>,
    mut healths: Query<&mut GelViscosity>,
    mut commands: Commands,
) {
    if let Ok(mut health) = healths.get_mut(damage.victim) {
        if damage.damage > 0.0 && health.value <= 0.0 {
            commands.trigger(Kill {
                victim: damage.victim,
            });
        }

        health.value -= damage.damage;
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn update_damage_numbers(
    damage: On<Damage>,
    mut damage_numbers: Query<Entity, With<DamageNumbers>>,
    hit_object: Query<Option<&Name>, With<GelViscosity>>,
    mut commands: Commands,
) {
    let hit_object_name = ok_or_return!(hit_object.get(damage.victim));
    let hit_object_name = hit_object_name
        .map(|name| name.as_str())
        .unwrap_or("Object");

    for damage_numbers in damage_numbers.iter_mut() {
        commands.spawn((
            TextSpan(format!("\n{hit_object_name}: {:.2}", damage.damage)),
            TextColor(Color::mix(
                &Color::from(css::RED),
                &Color::from(css::GREEN),
                damage.fray_modifier.clamp(0.0, 1.0),
            )),
            ChildOf(damage_numbers),
        ));
    }
}

#[add_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn initialize_weapon_sets(
    mut commands: Commands,
    mut weapon_sets: Query<(Entity, &mut Weapons), With<UninitializedWeaponSet>>,
) {
    for (entity, mut weapon_set) in weapon_sets.iter_mut() {
        if weapon_set.weapons.is_empty() {
            continue;
        }
        if weapon_set.active_weapon.is_none() {
            weapon_set.active_weapon = Some(weapon_set.weapons[0]);
        }
        for weapon in weapon_set.weapons.iter() {
            if Some(*weapon) == weapon_set.active_weapon {
                show_weapon(&mut commands, *weapon);
            } else {
                hide_weapon(&mut commands, *weapon);
            }
        }
        commands.entity(entity).remove::<UninitializedWeaponSet>();
    }
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn switch_weapon_next(
    next: On<JustPressed<NextWeapon>>,
    mut commands: Commands,
    weapons: Query<&WeaponOf>,
    mut weapon_sets: Query<&mut Weapons>,
) -> Result {
    let old_weapon = next.input;
    let weapon_of = weapons.get(old_weapon)?;
    let mut weapon_set = weapon_sets.get_mut(weapon_of.0)?;
    hide_weapon(&mut commands, old_weapon);
    let old_weapon_index = weapon_set
        .weapons
        .iter()
        .position(|&w| w == old_weapon)
        .ok_or("Current weapon not found in weapon set")?;

    let new_weapon_index = (old_weapon_index + 1) % weapon_set.weapons.len();
    let new_weapon = weapon_set.weapons[new_weapon_index];
    weapon_set.active_weapon = Some(new_weapon);
    show_weapon(&mut commands, new_weapon);
    Ok(())
}

#[add_observer(plugin = PlayerControllerPlugin)]
fn switch_weapon_prev(
    prev: On<JustPressed<PrevWeapon>>,
    mut commands: Commands,
    weapons: Query<&WeaponOf>,
    mut weapon_sets: Query<&mut Weapons>,
) -> Result {
    let old_weapon = prev.input;
    let weapon_of = weapons.get(old_weapon)?;
    let mut weapon_set = weapon_sets.get_mut(weapon_of.0)?;
    hide_weapon(&mut commands, old_weapon);
    let old_weapon_index = weapon_set
        .weapons
        .iter()
        .position(|&w| w == old_weapon)
        .ok_or("Current weapon not found in weapon set")?;

    let new_weapon_index =
        (old_weapon_index + weapon_set.weapons.len() - 1) % weapon_set.weapons.len();
    let new_weapon = weapon_set.weapons[new_weapon_index];
    weapon_set.active_weapon = Some(new_weapon);
    show_weapon(&mut commands, new_weapon);
    Ok(())
}

fn hide_weapon(commands: &mut Commands, weapon: Entity) {
    commands
        .entity(weapon)
        .remove::<ActiveWeapon>()
        .insert(Visibility::Hidden)
        .insert(InputDisabled);
}

fn show_weapon(commands: &mut Commands, weapon: Entity) {
    commands
        .entity(weapon)
        .insert(ActiveWeapon)
        .insert(Visibility::Inherited)
        .remove::<InputDisabled>();
}

#[derive(Action)]
pub struct Attack;

#[derive(Action)]
pub struct NextWeapon;

#[derive(Action)]
pub struct PrevWeapon;
