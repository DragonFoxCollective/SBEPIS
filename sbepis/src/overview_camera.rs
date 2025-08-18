use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_butler::*;

use crate::camera::PlayerCamera;
use crate::menus::{
    Menu, MenuActivated, MenuActivatedSet, MenuDeactivated, MenuDeactivatedSet,
    MenuManipulationSet, MenuStack, MenuWithMouse,
};
use crate::prelude::*;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
pub struct OverviewCameraPlugin;

#[add_plugin(to_plugin = OverviewCameraPlugin, init = PanOrbitCameraPlugin)]
use bevy_panorbit_camera::PanOrbitCameraPlugin;

#[derive(Component)]
pub struct OverviewCamera;

#[add_system(
	plugin = OverviewCameraPlugin, schedule = Startup,
)]
fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Overview Camera"),
        Camera {
            is_active: false,
            ..default()
        },
        Transform::from_xyz(4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        bevy_panorbit_camera::PanOrbitCamera {
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Left,
            modifier_pan: Some(KeyCode::ShiftLeft),
            reversed_zoom: true,
            ..default()
        },
        OverviewCamera,
        Menu,
        MenuWithMouse,
        #[cfg(feature = "inspector")]
        bevy_inspector_egui::bevy_egui::PrimaryEguiContext,
    ));
}

#[add_system(
	plugin = OverviewCameraPlugin, schedule = Update,
	run_if = input_just_pressed(KeyCode::Tab),
	in_set = MenuManipulationSet,
)]
fn toggle_camera(
    mut menu_stack: ResMut<MenuStack>,
    overview_camera: Query<Entity, With<OverviewCamera>>,
) -> Result {
    let overview_camera = overview_camera.single()?;
    menu_stack.toggle(overview_camera);
    Ok(())
}

#[add_system(
	plugin = OverviewCameraPlugin, schedule = Update,
	after = MenuActivatedSet,
)]
fn enable_overview_camera(
    mut ev_activated: EventReader<MenuActivated>,
    mut overview_camera: Query<&mut Camera, (With<OverviewCamera>, Without<PlayerCamera>)>,
    mut player_camera: Query<&mut Camera, (With<PlayerCamera>, Without<OverviewCamera>)>,
) -> Result {
    for MenuActivated(menu) in ev_activated.read() {
        if overview_camera.get(*menu).is_ok() {
            for mut overview_camera in overview_camera.iter_mut() {
                overview_camera.is_active = true;
            }
            for mut player_camera in player_camera.iter_mut() {
                player_camera.is_active = false;
            }
        }
    }
    Ok(())
}

#[add_system(
	plugin = OverviewCameraPlugin, schedule = Update,
	after = MenuDeactivatedSet,
)]
fn disable_overview_camera(
    mut ev_deactivated: EventReader<MenuDeactivated>,
    mut overview_camera: Query<&mut Camera, (With<OverviewCamera>, Without<PlayerCamera>)>,
    mut player_camera: Query<&mut Camera, (With<PlayerCamera>, Without<OverviewCamera>)>,
) -> Result {
    for MenuDeactivated(menu) in ev_deactivated.read() {
        if overview_camera.get(*menu).is_ok() {
            for mut overview_camera in overview_camera.iter_mut() {
                overview_camera.is_active = false;
            }
            for mut player_camera in player_camera.iter_mut() {
                player_camera.is_active = true;
            }
        }
    }
    Ok(())
}
