use bevy::prelude::*;
use bevy_butler::*;
use return_ok::ok_or_return;

use crate::camera::PlayerCameraNode;
use crate::player_controller::PlayerControllerPlugin;
use crate::prelude::PlayerBody;

#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub recovery_rate: f32,
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn update_stamina(mut players: Query<&mut Stamina>, time: Res<Time>) {
    for mut stamina in players.iter_mut() {
        stamina.current =
            (stamina.current + stamina.recovery_rate * time.delta_secs()).clamp(0.0, stamina.max);
    }
}

#[derive(Component)]
pub struct StaminaBar;

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Startup,
)]
fn setup_stamina_bar(mut commands: Commands) {
    commands
        .spawn((
            PlayerCameraNode,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(20.0),
                        margin: UiRect {
                            bottom: Val::Px(30.0),
                            left: Val::Px(30.0),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                ))
                .with_child((
                    StaminaBar,
                    Text::new("Hubris"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
        });
}

#[add_system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn update_stamina_bar(
    staminas: Query<&Stamina, With<PlayerBody>>,
    mut stamina_bars: Query<&mut Node, With<StaminaBar>>,
) {
    let stamina = ok_or_return!(staminas.single());
    let mut stamina_bar = ok_or_return!(stamina_bars.single_mut());
    stamina_bar.width = Val::Percent(stamina.current / stamina.max * 100.0);
}
