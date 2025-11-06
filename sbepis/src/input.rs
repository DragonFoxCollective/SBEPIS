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

// TODO: there's gotta be a way to remove this now
pub trait ActionButtonEvent {
    type Action: Actionlike + Copy;
    type Button: Component + InputManagerReference;
    type Event: Event + InputManagerReference;
    fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> + 'static;
    fn action() -> Self::Action;
}
pub fn fire_action_button_events<'a, T: ActionButtonEvent<Event: Event<Trigger<'a>: Default>>>(
    input: Query<(Entity, &ActionState<T::Action>)>,
    buttons: Query<(&T::Button, &Interaction), Changed<Interaction>>,
    mut commands: Commands,
    mut system: Local<Option<SystemId<In<Entity>, ()>>>,
) -> Result {
    if system.is_none() {
        *system = Some(commands.register_system(T::make_event_system().pipe(
            |In(ev): In<T::Event>, mut commands: Commands| {
                commands.trigger(ev);
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

#[derive(EntityEvent)]
pub struct JustPressed<T: Actionlike> {
    #[event_target]
    pub input_manager: Entity,
    pub action: T,
}
