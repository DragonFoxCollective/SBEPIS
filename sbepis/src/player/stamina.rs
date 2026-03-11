use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use return_ok::ok_or_return;

use crate::player::PlayerControllerPlugin;
use crate::prelude::*;

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub recovery_rate: f32,
}

impl Stamina {
    pub fn checked_sub_mut(&mut self, stamina: f32) -> bool {
        if self.current >= stamina {
            self.current -= stamina;
            true
        } else {
            false
        }
    }
}

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_stamina(mut players: Query<&mut Stamina>, time: Res<Time>) {
    for mut stamina in players.iter_mut() {
        stamina.current =
            (stamina.current + stamina.recovery_rate * time.delta_secs()).clamp(0.0, stamina.max);
    }
}

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
pub struct StaminaBar;

#[auto_system(plugin = PlayerControllerPlugin, schedule = OnEnter(GameState::InGame))]
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

#[auto_system(plugin = PlayerControllerPlugin, schedule = Update)]
fn update_stamina_bar(
    staminas: Query<&Stamina, With<Player>>,
    mut stamina_bars: Query<&mut Node, With<StaminaBar>>,
) {
    let stamina = ok_or_return!(staminas.single());
    for mut stamina_bar in stamina_bars.iter_mut() {
        stamina_bar.width = Val::Percent(stamina.current / stamina.max * 100.0);
    }
}
