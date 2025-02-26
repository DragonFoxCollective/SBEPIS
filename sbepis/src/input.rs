use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub fn input_manager_bundle<Action: Actionlike>(
	input_map: InputMap<Action>,
	start_enabled: bool,
) -> InputManagerBundle<Action> {
	let mut action_state: ActionState<Action> = default();
	if !start_enabled {
		action_state.disable();
	}
	InputManagerBundle::<Action> {
		input_map,
		action_state,
	}
}

pub trait ActionButtonEvent {
	type Action: Actionlike + Copy;
	type Button: Component + InputManagerReference;
	type Event: Event + InputManagerReference;
	fn make_event_system() -> impl IntoSystem<In<Entity>, Self::Event, ()> + 'static;
	fn action() -> Self::Action;
}
pub fn fire_action_button_events<T: ActionButtonEvent>(
	input: Query<(Entity, &ActionState<T::Action>)>,
	buttons: Query<(&T::Button, &Interaction), Changed<Interaction>>,
	mut commands: Commands,
	mut system: Local<Option<SystemId<In<Entity>, ()>>>,
) {
	if system.is_none() {
		*system = Some(commands.register_system(T::make_event_system().pipe(
			|In(ev): In<T::Event>, mut ev_action: EventWriter<T::Event>| {
				ev_action.send(ev);
			},
		)));
	}
	let system = system.unwrap();

	input
		.iter()
		.filter(|(_, input)| input.just_pressed(&T::action()))
		.for_each(|(entity, _)| {
			commands.run_system_with_input(system, entity);
		});

	buttons
		.iter()
		.filter(|(_, &interaction)| interaction == Interaction::Pressed)
		.for_each(|(button, _)| {
			commands.run_system_with_input(system, button.input_manager());
		});
}

pub trait InputManagerReference {
	fn input_manager(&self) -> Entity;
}

pub trait MapsToEvent<Event> {
	fn make_event(&self) -> Event;
}
pub fn map_event<EventA: Event + MapsToEvent<EventB>, EventB: Event>(
	mut ev_a: EventReader<EventA>,
	mut ev_b: EventWriter<EventB>,
) {
	for ev in ev_a.read() {
		ev_b.send(ev.make_event());
	}
}
pub fn map_action_to_event<Action: Actionlike + MapsToEvent<EventB>, EventB: Event>(
	input: Query<(Entity, &ActionState<Action>)>,
	mut ev_b: EventWriter<EventB>,
) {
	input
		.iter()
		.flat_map(|(_, input)| input.get_just_pressed())
		.for_each(|action| {
			ev_b.send(action.make_event());
		});
}

pub fn button_just_pressed<T: Actionlike + Copy>(
	action: T,
) -> impl Fn(Query<&ActionState<T>>) -> bool {
	move |input: Query<&ActionState<T>>| {
		if let Some(input) = input.iter().find(|input| !input.disabled()) {
			input.just_pressed(&action)
		} else {
			false
		}
	}
}

pub fn button_just_released<T: Actionlike + Copy>(
	action: T,
) -> impl Fn(Query<&ActionState<T>>) -> bool {
	move |input: Query<&ActionState<T>>| {
		if let Some(input) = input.iter().find(|input| !input.disabled()) {
			input.just_released(&action)
		} else {
			false
		}
	}
}
