use std::fmt::{self, Display, Formatter};

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::Collider;
use rand::Rng;
use rand::distr::weighted::WeightedIndex;
use rand::distr::{Distribution, StandardUniform};
use return_ok::some_or_return_ok;
use uuid::Uuid;

use crate::entity::Kill;
use crate::gridbox_material;
use crate::inventory::{ChangeInventory, Inventory, Item};
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
    FindMyPages,
}
impl QuestType {
    pub fn is_completed(&self) -> bool {
        match self {
            QuestType::Fetch { done } => *done,
            QuestType::Kill { amount, done } => *done >= *amount,
            QuestType::FindMyPages => false,
        }
    }

    pub fn min_progress(&self) -> u32 {
        match self {
            QuestType::Fetch { .. } => 0,
            QuestType::Kill { .. } => 0,
            QuestType::FindMyPages => 0,
        }
    }

    pub fn max_progress(&self) -> u32 {
        match self {
            QuestType::Fetch { .. } => 1,
            QuestType::Kill { amount, .. } => *amount,
            QuestType::FindMyPages => 8,
        }
    }

    pub fn progress(&self) -> u32 {
        match self {
            QuestType::Fetch { done } => *done as u32,
            QuestType::Kill { done, amount } => (*done).min(*amount),
            QuestType::FindMyPages => 0,
        }
    }

    pub fn progress_range(&self) -> std::ops::Range<f32> {
        self.min_progress() as f32..self.max_progress() as f32
    }
}
impl Distribution<QuestType> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuestType {
        let dist = WeightedIndex::new([10, 10, 1]).unwrap();
        match dist.sample(rng) {
            0 => QuestType::Fetch { done: false },
            1 => QuestType::Kill {
                amount: rng.random_range(1..=5),
                done: 0,
            },
            2 => QuestType::FindMyPages,
            _ => unreachable!(),
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
            QuestType::FindMyPages => Quest {
                id: QuestId::new(),
                name: "Find my pages.".to_string(),
                quest_type,
                description: "Find my pages.".to_string(),
            },
        }
    }
}

#[derive(Component, Default, Reflect)]
pub struct QuestGiver {
    pub given_quest: Option<QuestId>,
    quest_marker: Option<Entity>,
}

#[derive(EntityEvent, Clone)]
pub struct AcceptQuest {
    #[event_target]
    pub quest_proposal: Entity,
    pub quest_id: QuestId,
}

/// Quest has been declined, and will end.
#[derive(EntityEvent, Clone)]
pub struct DeclineQuest {
    #[event_target]
    pub quest_proposal: Entity,
    pub quest_id: QuestId,
}

/// Quest has ended, either by being completed, declined, or something else.
#[derive(Event)]
pub struct EndQuest {
    pub quest_id: QuestId,
}

/// Quest has been completed successfully, and will end.
#[derive(Event, Clone)]
pub struct CompleteQuest {
    pub quest_id: QuestId,
}

#[add_observer(plugin = QuestingPlugin)]
fn complete_to_end(complete: On<CompleteQuest>, mut commands: Commands) {
    commands.trigger(EndQuest {
        quest_id: complete.quest_id,
    });
}
#[add_observer(plugin = QuestingPlugin)]
fn decline_to_end(decline: On<DeclineQuest>, mut commands: Commands) {
    commands.trigger(EndQuest {
        quest_id: decline.quest_id,
    });
}

#[add_observer(plugin = QuestingPlugin)]
fn complete_quest_if_done(
    interact: On<InteractWith<QuestGiver>>,
    mut commands: Commands,
    quests: Res<Quests>,
    quest_givers: Query<&QuestGiver>,
) -> Result {
    let quest_proposal = quest_givers.get(interact.entity)?;
    let quest_id = some_or_return_ok!(quest_proposal.given_quest);
    let quest = quests.0.get(&quest_id).ok_or("Unknown quest")?;
    if !quest.quest_type.is_completed() {
        return Ok(());
    }
    commands.trigger(CompleteQuest { quest_id });
    Ok(())
}

#[add_observer(plugin = QuestingPlugin)]
fn end_quest_if_giver_killed(
    kill: On<Kill>,
    mut commands: Commands,
    quest_givers: Query<&QuestGiver>,
) {
    if let Ok(quest_proposal) = quest_givers.get(kill.victim)
        && let Some(quest_id) = quest_proposal.given_quest
    {
        commands.trigger(EndQuest { quest_id });
    }
}

#[add_observer(plugin = QuestingPlugin)]
fn remove_quest(
    end: On<EndQuest>,
    mut quests: ResMut<Quests>,
    mut quest_givers: Query<&mut QuestGiver>,
) -> Result {
    quests.0.remove(&end.quest_id);

    let mut quest_giver = quest_givers
        .iter_mut()
        .find(|qg| qg.given_quest == Some(end.quest_id))
        .ok_or("Quest giver missing")?;
    quest_giver.given_quest = None;

    Ok(())
}

#[add_observer(plugin = QuestingPlugin)]
fn update_killed_imps(kill: On<Kill>, mut quests: ResMut<Quests>, imps: Query<(), With<Imp>>) {
    if imps.get(kill.victim).is_ok() {
        for (_, quest) in quests.0.iter_mut() {
            if let QuestType::Kill { done, .. } = &mut quest.quest_type {
                *done += 1;
            }
        }
    }
}

#[add_observer(plugin = QuestingPlugin)]
fn update_picked_up_items(
    _change: On<ChangeInventory>,
    inventories: Query<&Inventory>,
    mut quests: ResMut<Quests>,
) {
    let num_items = inventories.iter().map(|inv| inv.items.len()).sum::<usize>();
    for (_, quest) in quests.0.iter_mut() {
        if let QuestType::Fetch { done } = &mut quest.quest_type {
            *done = num_items > 0;
        }
    }
}

#[add_observer(plugin = QuestingPlugin)]
fn spawn_quest_drops(
    kill: On<Kill>,
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
    let num_items = items.iter().count();
    if num_items >= num_fetch_quests {
        return;
    }

    if let Ok(transform) = imps.get(kill.victim) {
        if rand::random() {
            return;
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
    }
}

#[add_observer(plugin = QuestingPlugin)]
fn consume_quest_drop(
    complete: On<CompleteQuest>,
    mut inventories: Query<&mut Inventory>,
    mut commands: Commands,
    quests: Res<Quests>,
) -> Result {
    let quest = quests.0.get(&complete.quest_id).ok_or("Unknown quest")?;
    if let QuestType::Fetch { .. } = &quest.quest_type
        && quest.quest_type.is_completed()
    {
        let mut inventory = inventories.single_mut()?;
        let item = inventory.items.pop().ok_or("No item to consume")?;
        commands.entity(item).despawn();
    }

    Ok(())
}
