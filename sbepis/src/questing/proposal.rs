use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::*;

use crate::dialogue::spawn_dialogue;
use crate::input::{ActionButtonEvent, InputManagerReference};
use crate::menus::*;
use crate::player_controller::camera_controls::InteractedWith;
use crate::questing::{
    InteractedWithQuestGiverSet, Quest, QuestAccepted, QuestAcceptedSet, QuestDeclined,
    QuestDeclinedSet, QuestGiver, QuestId, QuestingPlugin, Quests,
};

#[derive(Component)]
pub struct QuestProposal {
    pub quest_id: QuestId,
}

#[derive(Component)]
pub struct QuestProposalAccept {
    pub quest_proposal: Entity,
}
impl InputManagerReference for QuestProposalAccept {
    fn input_manager(&self) -> Entity {
        self.quest_proposal
    }
}
impl ActionButtonEvent for QuestProposalAccept {
    type Action = QuestProposalAction;
    type Button = Self;
    type Event = QuestAccepted;

    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> {
        IntoSystem::into_system(
            |In(quest_proposal): In<Entity>, quest_proposals: Query<&QuestProposal>| {
                let quest_id = quest_proposals.get(quest_proposal).unwrap().quest_id;
                Self::Event {
                    quest_proposal,
                    quest_id,
                }
            },
        )
    }

    fn action() -> Self::Action {
        Self::Action::Accept
    }
}

#[derive(Component)]
pub struct QuestProposalDecline {
    pub quest_proposal: Entity,
}
impl InputManagerReference for QuestProposalDecline {
    fn input_manager(&self) -> Entity {
        self.quest_proposal
    }
}
impl ActionButtonEvent for QuestProposalDecline {
    type Action = QuestProposalAction;
    type Button = Self;
    type Event = QuestDeclined;

    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> {
        IntoSystem::into_system(
            |In(quest_proposal): In<Entity>, quest_proposals: Query<&QuestProposal>| {
                let quest_id = quest_proposals.get(quest_proposal).unwrap().quest_id;
                Self::Event {
                    quest_proposal,
                    quest_id,
                }
            },
        )
    }

    fn action() -> Self::Action {
        Self::Action::Decline
    }
}

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestProposalAccept>,
	in_set = QuestAcceptedSet,
)]
#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestProposalDecline>,
	in_set = QuestDeclinedSet,
)]
use crate::input::fire_action_button_events;

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestDeclined>,
	after = QuestDeclinedSet,
	in_set = MenuManipulationSet,
)]
#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestAccepted>,
	after = QuestAcceptedSet,
	in_set = MenuManipulationSet,
)]
use crate::menus::close_menu_on_event;

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	generics = <QuestGiver>,
)]
use crate::prelude::interact_with;

#[add_system(
	plugin = QuestingPlugin, schedule = Update,
	after = InteractedWithQuestGiverSet::default(),
)]
fn propose_quest_if_none(
    mut ev_interact: EventReader<InteractedWith<QuestGiver>>,
    mut commands: Commands,
    mut quests: ResMut<Quests>,
    mut quest_givers: Query<&mut QuestGiver>,
    mut menu_stack: ResMut<MenuStack>,
) -> Result {
    for ev in ev_interact.read() {
        let mut quest_giver = quest_givers.get_mut(ev.0)?;
        if quest_giver.given_quest.is_some() {
            continue;
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
            QuestProposal { quest_id },
            InputMap::default()
                .with(QuestProposalAction::Accept, KeyCode::KeyE)
                .with(QuestProposalAction::Decline, KeyCode::Space),
        );
        dialogue.add_option(
            &mut commands,
            "Accept [E]".to_owned(),
            QuestProposalAccept {
                quest_proposal: dialogue.root,
            },
        );
        dialogue.add_option(
            &mut commands,
            "Decline [Space]".to_owned(),
            QuestProposalDecline {
                quest_proposal: dialogue.root,
            },
        );
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum QuestProposalAction {
    Accept,
    Decline,
}
impl Actionlike for QuestProposalAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            QuestProposalAction::Accept => InputControlKind::Button,
            QuestProposalAction::Decline => InputControlKind::Button,
        }
    }
}
