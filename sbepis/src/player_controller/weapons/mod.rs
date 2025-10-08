use bevy::color::palettes::css;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::na::Vector3;
use bevy_rapier3d::parry::shape::{self, SharedShape};
use bevy_rapier3d::prelude::*;
use return_ok::ok_or_return;

use crate::entity::{EntityKilledSet, GelViscosity, Kill};
use crate::fray::FrayMusic;
use crate::input::button_just_pressed;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::util::{QuaternionEx, find_in_ancestors};

pub mod hammer;
pub mod rifle;
pub mod sword;

#[derive(Message)]
#[add_message(plugin = PlayerControllerPlugin)]
pub struct Hit {
    pub victim: Entity,
    pub perpetrator: Entity,
    pub allies: EntityHashSet,
    pub damage: f32,
    pub fray_modifier: f32,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HitSystems;

#[derive(Message)]
#[add_message(plugin = PlayerControllerPlugin)]
pub struct Damage {
    pub victim: Entity,
    pub damage: f32,
    pub fray_modifier: f32,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityDamagedSet;

#[derive(Component)]
pub struct DamageNumbers;

#[derive(Component)]
pub struct WeaponSet {
    pub weapons: Vec<Entity>,
    pub active_weapon: usize,
}

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

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Use),
)]
fn attack(mut weapons: Query<(&WeaponAnimation, &mut AnimationPlayer), With<ActiveWeapon>>) {
    for (animation, mut animation_player) in weapons.iter_mut() {
        if let Some(animation) = animation_player.animation_mut(animation.0) {
            if animation.is_finished() {
                animation.replay();
            }
        } else {
            animation_player.stop_all();
            animation_player.play(animation.0);
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
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

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = HitSystems,
)]
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
    mut hit: MessageWriter<Hit>,
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
                hit.write(Hit {
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

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = HitSystems,
	in_set = EntityDamagedSet,
)]
fn hit_to_damage(
    parents: Query<&ChildOf>,
    healths: Query<Entity, With<GelViscosity>>,
    mut hit: MessageReader<Hit>,
    mut damage: MessageWriter<Damage>,
) {
    for event in hit.read() {
        let victim = find_in_ancestors(event.victim, &healths, &parents).unwrap_or(event.victim);
        if !event.allies.contains(&victim) {
            damage.write(Damage {
                victim,
                damage: event.damage,
                fray_modifier: event.fray_modifier,
            });
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = EntityDamagedSet,
	in_set = EntityKilledSet,
)]
fn deal_all_damage(
    mut hit: MessageReader<Damage>,
    mut kill: MessageWriter<Kill>,
    mut healths: Query<&mut GelViscosity>,
) {
    for event in hit.read() {
        if let Ok(mut health) = healths.get_mut(event.victim) {
            let damage = event.damage;

            if damage > 0.0 && health.value <= 0.0 {
                kill.write(Kill(event.victim));
            }

            health.value -= damage;
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = EntityDamagedSet,
)]
fn update_damage_numbers(
    mut hit: MessageReader<Damage>,
    mut damage_numbers: Query<Entity, With<DamageNumbers>>,
    hit_object: Query<Option<&Name>, With<GelViscosity>>,
    mut commands: Commands,
) {
    for event in hit.read() {
        let Ok(hit_object_name) = hit_object.get(event.victim) else {
            continue;
        };
        let hit_object_name = hit_object_name
            .map(|name| name.as_str())
            .unwrap_or("Object");

        let damage = event.damage;
        let fray_modifier = event.fray_modifier;
        for damage_numbers in damage_numbers.iter_mut() {
            commands.spawn((
                TextSpan(format!("\n{hit_object_name}: {damage:.2}")),
                TextColor(Color::mix(
                    &Color::from(css::RED),
                    &Color::from(css::GREEN),
                    fray_modifier.clamp(0.0, 1.0),
                )),
                ChildOf(damage_numbers),
            ));
        }
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn initialize_weapon_sets(
    mut commands: Commands,
    weapon_sets: Query<(Entity, &WeaponSet), With<UninitializedWeaponSet>>,
) {
    for (entity, weapon_set) in weapon_sets.iter() {
        for (index, weapon) in weapon_set.weapons.iter().enumerate() {
            if index == weapon_set.active_weapon {
                show_weapon(&mut commands, *weapon);
            } else {
                hide_weapon(&mut commands, *weapon);
            }
        }
        commands.entity(entity).remove::<UninitializedWeaponSet>();
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::NextWeapon),
)]
fn switch_weapon_next(mut commands: Commands, mut weapon_sets: Query<&mut WeaponSet>) {
    for mut weapon_set in weapon_sets.iter_mut() {
        let old_weapon = weapon_set.weapons[weapon_set.active_weapon];
        hide_weapon(&mut commands, old_weapon);
        weapon_set.active_weapon = (weapon_set.active_weapon + 1) % weapon_set.weapons.len();
        let new_weapon = weapon_set.weapons[weapon_set.active_weapon];
        show_weapon(&mut commands, new_weapon);
    }
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::PrevWeapon),
)]
fn switch_weapon_prev(mut commands: Commands, mut weapon_sets: Query<&mut WeaponSet>) {
    for mut weapon_set in weapon_sets.iter_mut() {
        let old_weapon = weapon_set.weapons[weapon_set.active_weapon];
        hide_weapon(&mut commands, old_weapon);
        weapon_set.active_weapon =
            (weapon_set.active_weapon + weapon_set.weapons.len() - 1) % weapon_set.weapons.len();
        let new_weapon = weapon_set.weapons[weapon_set.active_weapon];
        show_weapon(&mut commands, new_weapon);
    }
}

fn hide_weapon(commands: &mut Commands, weapon: Entity) {
    commands
        .entity(weapon)
        .remove::<ActiveWeapon>()
        .insert(Visibility::Hidden);
}

fn show_weapon(commands: &mut Commands, weapon: Entity) {
    commands
        .entity(weapon)
        .insert(ActiveWeapon)
        .insert(Visibility::Inherited);
}
