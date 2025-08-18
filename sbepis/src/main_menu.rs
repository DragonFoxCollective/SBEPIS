use bevy::prelude::*;
use bevy_butler::*;

use crate::prelude::*;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
struct MainMenuPlugin;

#[insert_state(plugin = MainMenuPlugin, init = GameState::MainMenu)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
#[states(scoped_entities)]
pub enum GameState {
    MainMenu,
    InGame,
}

#[add_system(plugin = MainMenuPlugin, schedule = OnEnter(GameState::MainMenu))]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            reflectance: 0.02,
            unlit: false,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        StateScoped(GameState::MainMenu),
    ));

    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        StateScoped(GameState::MainMenu),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameState::MainMenu),
    ));
}
