use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_butler::*;
use bevy_rapier3d::prelude::Collider;
use proposal::*;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use screen::QuestProgressUpdatedSet;
use uuid::Uuid;

use crate::entity::{EntityKilled, EntityKilledSet};
use crate::input::{InputManagerReference, MapsToEvent};
use crate::inventory::{Inventory, InventoryChanged, InventoryChangedSet, Item};
use crate::menus::*;
use crate::npcs::imp::Imp;
use crate::prelude::InteractedWithSet;
use crate::{Box, gridbox_material, some_or_return};

mod proposal;
mod quest_markers;
mod screen;

pub use quest_markers::SpawnQuestMarker;

pub struct QuestingPlugin;

#[butler_plugin(build(
	add_plugins(InputManagerMenuPlugin::<QuestProposalAction>::default()),
))]
impl Plugin for QuestingPlugin {
	fn build(&self, app: &mut App) {
		#[cfg(feature = "inspector")]
		app.register_type_data::<QuestId, bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl>();
	}
}

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
#[resource(plugin = QuestingPlugin)]
#[register_type(plugin = QuestingPlugin)]
pub struct Quests(pub HashMap<QuestId, Quest>);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Reflect)]
#[register_type(plugin = QuestingPlugin)]
pub struct QuestId(Uuid);
impl QuestId {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Uuid::new_v4())
	}
}
impl Display for QuestId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
#[cfg(feature = "inspector")]
impl bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive for QuestId {
	fn ui(
		&mut self,
		ui: &mut bevy_inspector_egui::egui::Ui,
		_options: &dyn std::any::Any,
		_id: bevy_inspector_egui::egui::Id,
		_env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
	) -> bool {
		ui.add_enabled_ui(false, |ui| {
			ui.text_edit_singleline(&mut self.0.to_string());
		});
		false
	}

	fn ui_readonly(
		&self,
		ui: &mut bevy_inspector_egui::egui::Ui,
		_options: &dyn std::any::Any,
		_id: bevy_inspector_egui::egui::Id,
		_env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
	) {
		ui.add_enabled_ui(false, |ui| {
			ui.text_edit_singleline(&mut self.0.to_string());
		});
	}
}

#[derive(Debug, Reflect)]
pub enum QuestType {
	Fetch { done: bool },
	Kill { amount: u32, done: u32 },
}
impl QuestType {
	pub fn is_completed(&self) -> bool {
		match self {
			QuestType::Fetch { done } => *done,
			QuestType::Kill { amount, done } => *done >= *amount,
		}
	}

	pub fn min_progress(&self) -> u32 {
		match self {
			QuestType::Fetch { .. } => 0,
			QuestType::Kill { .. } => 0,
		}
	}

	pub fn max_progress(&self) -> u32 {
		match self {
			QuestType::Fetch { .. } => 1,
			QuestType::Kill { amount, .. } => *amount,
		}
	}

	pub fn progress(&self) -> u32 {
		match self {
			QuestType::Fetch { done } => *done as u32,
			QuestType::Kill { done, amount } => (*done).min(*amount),
		}
	}

	pub fn progress_range(&self) -> std::ops::Range<f32> {
		self.min_progress() as f32..self.max_progress() as f32
	}
}
impl Distribution<QuestType> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuestType {
		match rng.gen_range(0..=1) {
			0 => QuestType::Fetch { done: false },
			_ => QuestType::Kill {
				amount: rng.gen_range(1..=5),
				done: 0,
			},
		}
	}
}

#[derive(Debug, Reflect)]
#[register_type(plugin = QuestingPlugin)]
pub struct Quest {
	pub id: QuestId,
	pub quest_type: QuestType,
	pub name: String,
	pub description: String,
}
impl Distribution<Quest> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Quest {
		let quest_type: QuestType = rng.r#gen();
		match quest_type {
			QuestType::Kill { amount, .. } => Quest {
				id: QuestId::new(),
				name: "Awesome Kill Quest".to_string(),
				quest_type,
				description: format!(
					"imps killed my grandma... pwease go take revenge on those darn imps for me... kill {amount}!!"
				),
			},
			QuestType::Fetch { .. } => Quest {
				id: QuestId::new(),
				name: "Awesome Fetch Quest".to_string(),
				quest_type,
				description: "imps stole my orange cube... pwease go get it back!!".to_string(),
			},
		}
	}
}

#[derive(Component, Default, Reflect)]
#[register_type(plugin = QuestingPlugin)]
pub struct QuestGiver {
	pub given_quest: Option<QuestId>,
	quest_marker: Option<Entity>,
}

#[event(plugin = QuestingPlugin, generics = QuestGiver)]
use crate::prelude::InteractedWith;

#[derive(Event)]
#[event(plugin = QuestingPlugin)]
pub struct QuestAccepted {
	pub quest_proposal: Entity,
	pub quest_id: QuestId,
}
impl InputManagerReference for QuestAccepted {
	fn input_manager(&self) -> Entity {
		self.quest_proposal
	}
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestAcceptedSet;

#[derive(Event, Clone)]
#[event(plugin = QuestingPlugin)]
pub struct QuestDeclined {
	pub quest_proposal: Entity,
	pub quest_id: QuestId,
}
impl InputManagerReference for QuestDeclined {
	fn input_manager(&self) -> Entity {
		self.quest_proposal
	}
}
impl MapsToEvent<QuestEnded> for QuestDeclined {
	fn make_event(&self) -> QuestEnded {
		QuestEnded(self.quest_id)
	}
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestDeclinedSet;

#[derive(Event)]
#[event(plugin = QuestingPlugin)]
pub struct QuestEnded(pub QuestId);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestEndedSet;

#[derive(Event, Clone)]
#[event(plugin = QuestingPlugin)]
pub struct QuestCompleted(pub QuestId);
impl MapsToEvent<QuestEnded> for QuestCompleted {
	fn make_event(&self) -> QuestEnded {
		QuestEnded(self.0)
	}
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestCompletedSet;

#[system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestDeclined, QuestEnded>,
	after = QuestDeclinedSet,
	in_set = QuestEndedSet,
)]
#[system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestCompleted, QuestEnded>,
	after = QuestCompletedSet,
	in_set = QuestEndedSet,
)]
use crate::input::map_event;

type InteractedWithQuestGiverSet = InteractedWithSet<QuestGiver>;

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = InteractedWithQuestGiverSet::default(),
	in_set = QuestCompletedSet,
)]
fn complete_quest_if_done(
	mut ev_interact: EventReader<InteractedWith<QuestGiver>>,
	mut ev_completed: EventWriter<QuestCompleted>,
	quests: Res<Quests>,
	quest_givers: Query<&QuestGiver>,
) {
	for ev in ev_interact.read() {
		let quest_proposal = quest_givers.get(ev.0).expect("Quest giver not found");
		let quest_id = some_or_return!(quest_proposal.given_quest);
		let quest = quests.0.get(&quest_id).expect("Unknown quest");
		if !quest.quest_type.is_completed() {
			return;
		}
		ev_completed.send(QuestCompleted(quest_id));
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn end_quest_if_giver_killed(
	mut ev_killed: EventReader<EntityKilled>,
	mut ev_ended: EventWriter<QuestEnded>,
	quest_givers: Query<&QuestGiver>,
) {
	for &EntityKilled(entity) in ev_killed.read() {
		if let Ok(quest_proposal) = quest_givers.get(entity) {
			if let Some(quest_id) = quest_proposal.given_quest {
				ev_ended.send(QuestEnded(quest_id));
			}
		}
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = end_quest_if_giver_killed,
)]
fn remove_quest(
	mut ev_ended: EventReader<QuestEnded>,
	mut quests: ResMut<Quests>,
	mut quest_givers: Query<&mut QuestGiver>,
) {
	for ev in ev_ended.read() {
		quests.0.remove(&ev.0);

		let mut quest_giver = quest_givers
			.iter_mut()
			.find(|qg| qg.given_quest == Some(ev.0))
			.expect("Quest giver missing");
		quest_giver.given_quest = None;
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet,
	in_set = QuestProgressUpdatedSet,
)]
fn update_killed_imps(
	mut ev_killed: EventReader<EntityKilled>,
	mut quests: ResMut<Quests>,
	imps: Query<(), With<Imp>>,
) {
	for EntityKilled(entity) in ev_killed.read() {
		if imps.get(*entity).is_ok() {
			for (_, quest) in quests.0.iter_mut() {
				if let QuestType::Kill { done, .. } = &mut quest.quest_type {
					*done += 1;
				}
			}
		}
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = InventoryChangedSet,
	in_set = QuestProgressUpdatedSet,
	run_if = on_event::<InventoryChanged>,
)]
fn update_picked_up_items(inventories: Query<&Inventory>, mut quests: ResMut<Quests>) {
	let num_items = inventories.iter().map(|inv| inv.items.len()).sum::<usize>();
	for (_, quest) in quests.0.iter_mut() {
		if let QuestType::Fetch { done } = &mut quest.quest_type {
			*done = num_items > 0;
		}
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn spawn_quest_drops(
	mut ev_killed: EventReader<EntityKilled>,
	mut commands: Commands,
	quests: Res<Quests>,
	imps: Query<&Transform, With<Imp>>,
	items: Query<&Item>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let num_fetch_quests = quests
		.0
		.values()
		.filter(|quest| matches!(quest.quest_type, QuestType::Fetch { .. }))
		.count();
	let mut num_items = items.iter().count();

	for EntityKilled(entity) in ev_killed.read() {
		if num_items >= num_fetch_quests {
			break;
		}

		if let Ok(transform) = imps.get(*entity) {
			if rand::random() {
				continue;
			}

			commands.spawn((
				Transform::from_translation(transform.translation + Vec3::Y * 0.2),
				Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(0.2)))),
				MeshMaterial3d(gridbox_material("orange", &mut materials, &asset_server)),
				Box,
				Collider::cuboid(0.1, 0.1, 0.1),
				Item {
					icon: asset_server.load("item.png"),
				},
			));
			num_items += 1;
		}
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = QuestCompletedSet,
)]
fn consume_quest_drop(
	mut ev_completed: EventReader<QuestCompleted>,
	mut inventories: Query<&mut Inventory>,
	mut commands: Commands,
	quests: Res<Quests>,
) {
	for QuestCompleted(quest_id) in ev_completed.read() {
		let quest = quests.0.get(quest_id).expect("Unknown quest");
		if let QuestType::Fetch { .. } = &quest.quest_type {
			if quest.quest_type.is_completed() {
				let mut inventory = inventories.single_mut();
				let item = inventory.items.pop().expect("No item to consume");
				commands.entity(item).despawn_recursive();
			}
		}
	}
}
