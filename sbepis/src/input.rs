use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub fn input_manager_bundle<Action: Actionlike>(
    input_map: InputMap<Action>,
    start_enabled: bool,
) -> impl Bundle {
    let mut action_state: ActionState<Action> = default();
    if !start_enabled {
        action_state.disable();
    }
    (input_map, action_state)
}

pub trait ActionButtonMessage {
    type Action: Actionlike + Copy;
    type Button: Component + InputManagerReference;
    type Message: Message + InputManagerReference;
    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Message, ()> + 'static;
    fn action() -> Self::Action;
}
pub fn fire_action_button_messages<T: ActionButtonMessage>(
    input: Query<(Entity, &ActionState<T::Action>)>,
    buttons: Query<(&T::Button, &Interaction), Changed<Interaction>>,
    mut commands: Commands,
    mut system: Local<Option<SystemId<In<Entity>, ()>>>,
) -> Result {
    if system.is_none() {
        *system = Some(commands.register_system(T::make_event_system().pipe(
            |In(ev): In<T::Message>, mut actions: MessageWriter<T::Message>| {
                actions.write(ev);
            },
        )));
    }
    let system = system.ok_or("System not registered")?;

    input
        .iter()
        .filter(|(_, input)| input.just_pressed(&T::action()))
        .for_each(|(entity, _)| {
            commands.run_system_with(system, entity);
        });

    buttons
        .iter()
        .filter(|&(_, &interaction)| interaction == Interaction::Pressed)
        .for_each(|(button, _)| {
            commands.run_system_with(system, button.input_manager());
        });

    Ok(())
}

pub trait InputManagerReference {
    fn input_manager(&self) -> Entity;
}

pub trait MapsToMessage<Message> {
    fn make_event(&self) -> Message;
}
pub fn map_event<MessageA: Message + MapsToMessage<MessageB>, MessageB: Message>(
    mut message_a: MessageReader<MessageA>,
    mut message_b: MessageWriter<MessageB>,
) {
    for ev in message_a.read() {
        message_b.write(ev.make_event());
    }
}
pub fn map_action_to_event<Action: Actionlike + MapsToMessage<MessageB>, MessageB: Message>(
    input: Query<(Entity, &ActionState<Action>)>,
    mut message_b: MessageWriter<MessageB>,
) {
    input
        .iter()
        .flat_map(|(_, input)| input.get_just_pressed())
        .for_each(|action| {
            message_b.write(action.make_event());
        });
}

const UNIVERSAL_DEADZONE: f32 = 0.05;

pub fn button_pressed<T: Actionlike + Copy>(input: &ActionState<T>, action: &T) -> bool {
    match action.input_control_kind() {
        InputControlKind::Button => input.pressed(action),
        InputControlKind::Axis => input.value(action) >= UNIVERSAL_DEADZONE,
        InputControlKind::DualAxis => input.axis_pair(action).length() >= UNIVERSAL_DEADZONE,
        InputControlKind::TripleAxis => input.axis_triple(action).length() >= UNIVERSAL_DEADZONE,
    }
}

pub fn button_is_pressed<T: Actionlike + Copy>(
    action: T,
) -> impl Fn(Query<&ActionState<T>>) -> bool {
    move |input: Query<&ActionState<T>>| {
        if let Some(input) = input.iter().find(|input| !input.disabled()) {
            button_pressed(input, &action)
        } else {
            false
        }
    }
}

pub fn button_is_released<T: Actionlike + Copy>(
    action: T,
) -> impl Fn(Query<&ActionState<T>>) -> bool {
    move |input: Query<&ActionState<T>>| {
        if let Some(input) = input.iter().find(|input| !input.disabled()) {
            !button_pressed(input, &action)
        } else {
            true
        }
    }
}

pub fn button_just_pressed<T: Actionlike + Copy>(
    action: T,
) -> impl Fn(Query<&ActionState<T>>, Local<bool>) -> bool {
    move |input: Query<&ActionState<T>>, mut last: Local<bool>| {
        if let Some(input) = input.iter().find(|input| !input.disabled()) {
            let value = button_pressed(input, &action);
            let result = !*last && value;
            *last = value;
            result
        } else {
            false
        }
    }
}

pub fn button_just_released<T: Actionlike + Copy>(
    action: T,
) -> impl Fn(Query<&ActionState<T>>, Local<bool>) -> bool {
    move |input: Query<&ActionState<T>>, mut last: Local<bool>| {
        if let Some(input) = input.iter().find(|input| !input.disabled()) {
            let value = button_pressed(input, &action);
            let result = *last && !value;
            *last = value;
            result
        } else {
            false
        }
    }
}
