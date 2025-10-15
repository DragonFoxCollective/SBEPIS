use bevy::prelude::*;
use bevy_butler::*;
use bevy_pretty_nice_input::{Binding1D, input};
use bevy_pretty_nice_menus::MenuStack;

use crate::dialogue::{PickDialogueOption, spawn_dialogue};
use crate::player_controller::camera_controls::InteractWith;
use crate::questing::{AcceptQuest, DeclineQuest, Quest, QuestGiver, QuestingPlugin, Quests};

#[add_observer(plugin = QuestingPlugin, generics = <AcceptQuest>)]
#[add_observer(plugin = QuestingPlugin, generics = <DeclineQuest>)]
use bevy_pretty_nice_menus::close_menu_on_event;

#[add_observer(plugin = QuestingPlugin, generics = <QuestGiver>)]
use crate::prelude::interact_with;

#[add_observer(plugin = QuestingPlugin)]
fn propose_quest_if_none(
    interact: On<InteractWith<QuestGiver>>,
    mut commands: Commands,
    mut quests: ResMut<Quests>,
    mut quest_givers: Query<&mut QuestGiver>,
    mut menu_stack: ResMut<MenuStack>,
) -> Result {
    let mut quest_giver = quest_givers.get_mut(interact.entity)?;
    if quest_giver.given_quest.is_some() {
        return Ok(());
    }

    let quest: Quest = rand::random();
    let quest_id = quest.id;
    quests.0.insert(quest_id, quest);
    let quest = quests
        .0
        .get(&quest_id)
        .ok_or("Unknown quest even though we just inserted it")?;

    quest_giver.given_quest = Some(quest_id);

    let mut dialogue = spawn_dialogue(
        &mut commands,
        &mut menu_stack,
        format!("{}\n\n{}", quest.name, quest.description),
        (),
    );
    dialogue.add_option(
        &mut commands,
        "Accept [E]".to_owned(),
        input!(PickDialogueOption, [Binding1D::Key(KeyCode::KeyE)]),
        AcceptQuest {
            quest_proposal: dialogue.root,
            quest_id,
        },
    );
    dialogue.add_option(
        &mut commands,
        "Decline [Space]".to_owned(),
        input!(PickDialogueOption, [Binding1D::Key(KeyCode::Space)]),
        DeclineQuest {
            quest_proposal: dialogue.root,
            quest_id,
        },
    );

    Ok(())
}
