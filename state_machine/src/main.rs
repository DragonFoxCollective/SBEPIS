use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::utils::HashMap;
use itertools::Itertools as _;
use rand::distr::{Distribution, StandardUniform};
use uuid::Uuid;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, setup)
		.add_systems(
			Update,
			(
				update_side_panel.run_if(resource_changed::<StateTypes>),
				update_nodes,
			),
		)
		.add_systems(Update, quit_on_esc)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn((Camera2d,));

	let root = commands
		.spawn((Node {
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			..default()
		},))
		.id();

	let side_panel = commands
		.spawn((
			Node {
				width: Val::Px(200.0),
				height: Val::Percent(100.0),
				border: UiRect::all(Val::Px(5.0)).with_left(Val::Auto),
				padding: UiRect::all(Val::Px(10.0)),
				..default()
			},
			BackgroundColor(css::DARK_GRAY.into()),
			BorderColor(css::GRAY.into()),
			BorderRadius::right(Val::Px(10.0)),
		))
		.set_parent(root)
		.id();

	let _side_panel_text = commands
		.spawn((
			SidePanelText,
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				..default()
			},
		))
		.set_parent(side_panel)
		.id();

	let main_space = commands
		.spawn((Node {
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			..default()
		},))
		.set_parent(root)
		.id();

	let state_type_1 = StateTypeData::new("State Type 1");
	let state_type_1_id = state_type_1.id.clone();
	let state_type_2 = StateTypeData::new("State Type 2");
	let state_type_2_id = state_type_2.id.clone();

	let mut state_types = StateTypes::default();
	state_types.insert(state_type_1);
	state_types.insert(state_type_2);
	commands.insert_resource(state_types);

	commands
		.spawn((
			Node {
				position_type: PositionType::Absolute,
				top: Val::Px(50.0),
				left: Val::Px(50.0),
				border: UiRect::all(Val::Px(10.0)),
				padding: UiRect::all(Val::Px(10.0)),
				flex_direction: FlexDirection::Column,
				..default()
			},
			BackgroundColor(css::MAROON.into()),
			BorderColor(css::RED.into()),
			BorderRadius::all(Val::Px(10.0)),
			State(vec![
				StateValue {
					state: state_type_1_id,
					value: false,
				},
				StateValue {
					state: state_type_2_id,
					value: true,
				},
			]),
		))
		.set_parent(main_space);
}

#[derive(Resource, Debug, Default)]
pub struct StateTypes(HashMap<StateType, StateTypeData>);

impl StateTypes {
	pub fn insert(&mut self, state_type: StateTypeData) {
		self.0.insert(state_type.id.clone(), state_type);
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateType(Uuid);

impl Distribution<StateType> for StandardUniform {
	fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> StateType {
		StateType(Uuid::from_u128(rng.random()))
	}
}

#[derive(Debug)]
pub struct StateTypeData {
	pub id: StateType,
	pub name: String,
}

impl StateTypeData {
	pub fn new(name: &str) -> Self {
		Self {
			id: rand::random(),
			name: name.to_string(),
		}
	}
}

#[derive(Clone, Debug)]
pub struct StateValue {
	pub state: StateType,
	pub value: bool,
}

#[derive(Component, Clone, Debug)]
pub struct State(pub Vec<StateValue>);

#[derive(Component)]
#[require(Text)]
pub struct SidePanelText;

fn update_side_panel(
	mut side_panel: Query<&mut Text, With<SidePanelText>>,
	state_types: Res<StateTypes>,
) {
	for mut text in side_panel.iter_mut() {
		text.0 = state_types
			.0
			.iter()
			.map(|state_type| state_type.1.name.clone())
			.sorted()
			.join("\n");
	}
}

fn update_nodes(
	nodes: Query<(Entity, &State), Changed<State>>,
	mut commands: Commands,
	state_types: Res<StateTypes>,
) {
	for (node, state) in nodes.iter() {
		commands.entity(node).despawn_descendants();

		for (state_name, state_value) in state
			.0
			.iter()
			.map(|value| (state_types.0.get(&value.state).unwrap().name.clone(), value))
			.sorted_by_key(|(name, _)| name.clone())
		{
			let text = commands
				.spawn((
					Text(state_name),
					Node::default(),
					TextColor(if state_value.value {
						css::GREEN.into()
					} else {
						css::RED.into()
					}),
				))
				.id();

			let connector = commands
				.spawn((
					Node {
						width: Val::Px(15.0),
						height: Val::Px(15.0),
						border: UiRect::all(Val::Px(3.0)),
						..default()
					},
					BackgroundColor(if state_value.value {
						css::RED.into()
					} else {
						css::GREEN.into()
					}),
					BorderRadius::all(Val::Percent(100.0)),
					BorderColor(css::BLACK.into()),
				))
				.id();

			commands
				.spawn((Node {
					align_items: AlignItems::Center,
					..default()
				},))
				.set_parent(node)
				.add_child(text)
				.add_child(connector);
		}
	}
}

fn quit_on_esc(mut exit: EventWriter<AppExit>, keyboard_input: Res<ButtonInput<KeyCode>>) {
	if keyboard_input.just_pressed(KeyCode::Escape) {
		exit.send(AppExit::Success);
	}
}
