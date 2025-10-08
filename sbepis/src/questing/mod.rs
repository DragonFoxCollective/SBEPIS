use std::fmt::{self, Display, Formatter};

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::Collider;
use proposal::*;
use rand::Rng;
use rand::distr::{Distribution, StandardUniform};
use return_ok::some_or_continue;
use screen::QuestProgressUpdatedSet;
use uuid::Uuid;

use crate::entity::{EntityKilledSet, Kill};
use crate::gridbox_material;
use crate::input::{InputManagerReference, MapsToMessage};
use crate::inventory::{ChangeInventory, Inventory, InventoryChangedSet, Item};
use crate::main_bundles::Box;
use crate::npcs::imp::Imp;
use crate::prelude::*;

mod proposal;
mod quest_markers;
mod screen;

pub use quest_markers::SpawnQuestMarker;

#[add_plugin(to_plugin = SbepisPlugin)]
pub struct QuestingPlugin;
#[butler_plugin]
impl Plugin for QuestingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "inspector")]
		app.register_type_data::<QuestId, bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl>();
    }
}

#[add_plugin(to_plugin = QuestingPlugin, generics = <QuestProposalAction>)]
use crate::menus::InputManagerMenuPlugin;

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
#[insert_resource(plugin = QuestingPlugin)]
pub struct Quests(pub HashMap<QuestId, Quest>);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Reflect)]
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
impl Distribution<QuestType> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuestType {
        match rng.random_range(0..=1) {
            0 => QuestType::Fetch { done: false },
            _ => QuestType::Kill {
                amount: rng.random_range(1..=5),
                done: 0,
            },
        }
    }
}

#[derive(Debug, Reflect)]
pub struct Quest {
    pub id: QuestId,
    pub quest_type: QuestType,
    pub name: String,
    pub description: String,
}
impl Distribution<Quest> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Quest {
        let quest_type: QuestType = rng.random();
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
pub struct QuestGiver {
    pub given_quest: Option<QuestId>,
    quest_marker: Option<Entity>,
}

#[add_message(plugin = QuestingPlugin, generics = <QuestGiver>)]
use crate::prelude::InteractWith;

#[derive(Message)]
#[add_message(plugin = QuestingPlugin)]
pub struct AcceptQuest {
    pub quest_proposal: Entity,
    pub quest_id: QuestId,
}
impl InputManagerReference for AcceptQuest {
    fn input_manager(&self) -> Entity {
        self.quest_proposal
    }
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AcceptQuestSystems;

#[derive(Message, Clone)]
#[add_message(plugin = QuestingPlugin)]
pub struct QuestDeclined {
    pub quest_proposal: Entity,
    pub quest_id: QuestId,
}
impl InputManagerReference for QuestDeclined {
    fn input_manager(&self) -> Entity {
        self.quest_proposal
    }
}
impl MapsToMessage<EndQuest> for QuestDeclined {
    fn make_event(&self) -> EndQuest {
        EndQuest(self.quest_id)
    }
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeclineQuestSystems;

/// Quest has ended, either by being completed or declined.
#[derive(Message)]
#[add_message(plugin = QuestingPlugin)]
pub struct EndQuest(pub QuestId);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EndQuestSystems;

#[derive(Message, Clone)]
#[add_message(plugin = QuestingPlugin)]
pub struct CompleteQuest(pub QuestId);
impl MapsToMessage<EndQuest> for CompleteQuest {
    fn make_event(&self) -> EndQuest {
        EndQuest(self.0)
    }
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompleteQuestSystems;

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestDeclined, EndQuest>,
	after = DeclineQuestSystems,
	in_set = EndQuestSystems,
)]
#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <CompleteQuest, EndQuest>,
	after = CompleteQuestSystems,
	in_set = EndQuestSystems,
)]
use crate::input::map_event;

type InteractedWithQuestGiverSet = InteractedWithSet<QuestGiver>;

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = InteractedWithQuestGiverSet::default(),
	in_set = CompleteQuestSystems,
)]
fn complete_quest_if_done(
    mut interact: MessageReader<InteractWith<QuestGiver>>,
    mut complete_quest: MessageWriter<CompleteQuest>,
    quests: Res<Quests>,
    quest_givers: Query<&QuestGiver>,
) -> Result {
    for ev in interact.read() {
        let quest_proposal = quest_givers.get(ev.0)?;
        let quest_id = some_or_continue!(quest_proposal.given_quest);
        let quest = quests.0.get(&quest_id).ok_or("Unknown quest")?;
        if !quest.quest_type.is_completed() {
            continue;
        }
        complete_quest.write(CompleteQuest(quest_id));
    }
    Ok(())
}

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn end_quest_if_giver_killed(
    mut kill: MessageReader<Kill>,
    mut end_quest: MessageWriter<EndQuest>,
    quest_givers: Query<&QuestGiver>,
) {
    for &Kill(entity) in kill.read() {
        if let Ok(quest_proposal) = quest_givers.get(entity)
            && let Some(quest_id) = quest_proposal.given_quest
        {
            end_quest.write(EndQuest(quest_id));
        }
    }
}

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = end_quest_if_giver_killed,
)]
fn remove_quest(
    mut end: MessageReader<EndQuest>,
    mut quests: ResMut<Quests>,
    mut quest_givers: Query<&mut QuestGiver>,
) -> Result {
    for ev in end.read() {
        quests.0.remove(&ev.0);

        let mut quest_giver = quest_givers
            .iter_mut()
            .find(|qg| qg.given_quest == Some(ev.0))
            .ok_or("Quest giver missing")?;
        quest_giver.given_quest = None;
    }
    Ok(())
}

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet,
	in_set = QuestProgressUpdatedSet,
)]
fn update_killed_imps(
    mut kill: MessageReader<Kill>,
    mut quests: ResMut<Quests>,
    imps: Query<(), With<Imp>>,
) {
    for Kill(entity) in kill.read() {
        if imps.get(*entity).is_ok() {
            for (_, quest) in quests.0.iter_mut() {
                if let QuestType::Kill { done, .. } = &mut quest.quest_type {
                    *done += 1;
                }
            }
        }
    }
}

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = InventoryChangedSet,
	in_set = QuestProgressUpdatedSet,
	run_if = on_message::<ChangeInventory>,
)]
fn update_picked_up_items(inventories: Query<&Inventory>, mut quests: ResMut<Quests>) {
    let num_items = inventories.iter().map(|inv| inv.items.len()).sum::<usize>();
    for (_, quest) in quests.0.iter_mut() {
        if let QuestType::Fetch { done } = &mut quest.quest_type {
            *done = num_items > 0;
        }
    }
}

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn spawn_quest_drops(
    mut kill: MessageReader<Kill>,
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

    for Kill(entity) in kill.read() {
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

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = CompleteQuestSystems,
)]
fn consume_quest_drop(
    mut complete: MessageReader<CompleteQuest>,
    mut inventories: Query<&mut Inventory>,
    mut commands: Commands,
    quests: Res<Quests>,
) -> Result {
    for CompleteQuest(quest_id) in complete.read() {
        let quest = quests.0.get(quest_id).ok_or("Unknown quest")?;
        if let QuestType::Fetch { .. } = &quest.quest_type
            && quest.quest_type.is_completed()
        {
            let mut inventory = inventories.single_mut()?;
            let item = inventory.items.pop().ok_or("No item to consume")?;
            commands.entity(item).despawn();
        }
    }
    Ok(())
}
