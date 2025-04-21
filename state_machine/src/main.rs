use bevy::color::palettes::css;
use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, setup)
		.add_systems(Update, quit_on_esc)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn((Camera2d,));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(50.0),
			left: Val::Px(50.0),
			width: Val::Px(200.0),
			height: Val::Px(200.0),
			border: UiRect::all(Val::Px(10.0)),
			..default()
		},
		BackgroundColor(css::MAROON.into()),
		BorderColor(css::RED.into()),
		BorderRadius::all(Val::Px(10.0)),
	));
}

fn quit_on_esc(mut exit: EventWriter<AppExit>, keyboard_input: Res<ButtonInput<KeyCode>>) {
	if keyboard_input.just_pressed(KeyCode::Escape) {
		exit.send(AppExit::Success);
	}
}
