use bevy_auto_plugin::prelude::*;

mod commands;
mod note_holder;
mod notes;
mod staff;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = crate::SbepisPlugin)]
pub struct PlayerCommandsPlugin;

#[auto_plugin(plugin = PlayerCommandsPlugin)]
fn build(app: &mut bevy::prelude::App) {
    app.add_observer(
        bevy_pretty_nice_menus::show_menu_on_action::<
            crate::player_controller::OpenStaff,
            staff::CommandStaff,
        >,
    );
    app.add_observer(bevy_pretty_nice_menus::close_menu_on_action::<staff::CloseStaff>);
}
