use std::time::Instant;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_butler::*;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap};
use leafwing_input_manager::{Actionlike, InputControlKind};
use return_ok::ok_or_return;

use crate::input::{InputManagerReference, JustPressed};
use crate::prelude::*;

#[add_plugin(to_plugin = SbepisPlugin)]
#[butler_plugin]
pub struct MenusPlugin;

#[add_plugin(to_plugin = MenusPlugin, generics = <CloseMenuAction>)]
pub struct InputManagerMenuPlugin<Action: Actionlike>(std::marker::PhantomData<Action>);
impl<Action: Actionlike + TypePath + bevy::reflect::GetTypeRegistration> Plugin
    for InputManagerMenuPlugin<Action>
{
    fn build(&self, app: &mut App) {
        app.register_type::<ActionState<Action>>()
            .register_type::<InputMap<Action>>()
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_observer(enable_input_managers::<Action>)
            .add_observer(disable_input_managers::<Action>);
    }
}
impl<Action: Actionlike> Default for InputManagerMenuPlugin<Action> {
    fn default() -> Self {
        Self(default())
    }
}

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct MenuWithInputManager;

#[derive(Component)]
pub struct MenuWithMouse;

#[derive(Component)]
pub struct MenuWithoutMouse;

#[derive(Component)]
pub struct MenuHidesWhenClosed;

#[derive(Component)]
pub struct MenuDespawnsWhenClosed;

#[derive(Resource, Default, Debug, Reflect)]
#[insert_resource(plugin = MenusPlugin)]
pub struct MenuStack {
    stack: Vec<Entity>,
    current: Option<Entity>,
}
impl MenuStack {
    pub fn push(&mut self, menu: Entity) {
        self.stack.push(menu);
        debug!("Pushed menu {menu:?}, stack is now {self:?}");
    }

    pub fn remove(&mut self, menu: Entity) {
        self.stack.retain(|&entity| entity != menu);
        debug!("Removed menu {menu:?}, stack is now {self:?}");
    }

    pub fn contains(&self, menu: Entity) -> bool {
        self.stack.contains(&menu)
    }

    pub fn toggle(&mut self, menu: Entity) {
        if self.contains(menu) {
            self.remove(menu);
        } else {
            self.push(menu);
        }
    }
}

#[derive(EntityEvent)]
pub struct ActivateMenu {
    #[event_target]
    pub menu: Entity,
}

#[derive(EntityEvent)]
pub struct DeactivateMenu {
    #[event_target]
    pub menu: Entity,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub struct CloseMenuAction;
impl Actionlike for CloseMenuAction {
    fn input_control_kind(&self) -> InputControlKind {
        InputControlKind::Button
    }
}
impl CloseMenuBinding for CloseMenuAction {
    type Action = Self;
    fn action() -> Self {
        Self
    }
}

/// This is the main sync point for changing the menu stack to activating/deactivating menus.
#[add_system(plugin = MenusPlugin, schedule = PostUpdate)]
fn activate_stack_current(mut menu_stack: If<ResMut<MenuStack>>, mut commands: Commands) -> Result {
    if !menu_stack.is_changed() {
        return Ok(());
    }

    if let Some(current) = menu_stack.current
        && menu_stack.stack.last() != Some(&current)
    {
        commands.trigger(DeactivateMenu { menu: current });
        menu_stack.current = None;
    }

    if menu_stack.current.is_none() && !menu_stack.stack.is_empty() {
        let new_current = *menu_stack
            .stack
            .last()
            .ok_or("Menu stack was empty (impossible)")?;
        menu_stack.current = Some(new_current);
        commands.trigger(ActivateMenu { menu: new_current });
    }

    Ok(())
}

#[add_observer(plugin = MenusPlugin)]
fn show_mouse(
    activate: On<ActivateMenu>,
    menus: Query<(), With<MenuWithMouse>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if menus.get(activate.menu).is_ok() {
        let mut cursor_options = ok_or_return!(cursor_options.single_mut());
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

#[add_observer(plugin = MenusPlugin)]
fn hide_mouse(
    activate: On<ActivateMenu>,
    menus: Query<(), With<MenuWithoutMouse>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if menus.get(activate.menu).is_ok() {
        let mut cursor_options = ok_or_return!(cursor_options.single_mut());
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}

fn enable_input_managers<Action: Actionlike>(
    activate: On<ActivateMenu>,
    mut menus: Query<&mut ActionState<Action>, With<MenuWithInputManager>>,
) {
    if let Ok(mut input_manager) = menus.get_mut(activate.menu) {
        input_manager.enable();

        // On the first frame of a new input manager, already held buttons
        // are "just pressed" so we need to clear them
        input_manager.tick(Instant::now(), Instant::now());
    }
}

fn disable_input_managers<Action: Actionlike>(
    deactivate: On<DeactivateMenu>,
    mut menus: Query<&mut ActionState<Action>, With<MenuWithInputManager>>,
) {
    if let Ok(mut input_manager) = menus.get_mut(deactivate.menu) {
        input_manager.disable();
    }
}

pub trait CloseMenuBinding {
    type Action: Actionlike + Copy;
    fn action() -> Self::Action;
}
pub trait OpenMenuBinding {
    type Action: Actionlike + Copy;
    type Menu: Component;
    fn action() -> Self::Action;
}

#[add_observer(plugin = MenusPlugin, generics = <CloseMenuAction>)]
pub fn close_menu_on_action<Binding: CloseMenuBinding>(
    pressed: On<JustPressed<Binding::Action>>,
    mut menu_stack: ResMut<MenuStack>,
) {
    menu_stack.remove(pressed.input_manager);
}

pub fn close_menu_on_event<Ev: Event + InputManagerReference>(
    input_manager: On<Ev>,
    mut menu_stack: ResMut<MenuStack>,
) {
    menu_stack.remove(input_manager.input_manager());
}

pub fn show_menu_on_action<Binding: OpenMenuBinding>(
    _: On<JustPressed<Binding::Action>>,
    mut menus: Query<Entity, With<Binding::Menu>>,
    mut menu_stack: ResMut<MenuStack>,
) -> Result {
    let menu = menus.single_mut()?;
    menu_stack.push(menu);
    Ok(())
}

#[add_observer(plugin = MenusPlugin)]
fn show_menus(
    activate: On<ActivateMenu>,
    mut menus: Query<&mut Visibility, With<MenuHidesWhenClosed>>,
) {
    if let Ok(mut visibility) = menus.get_mut(activate.menu) {
        *visibility = Visibility::Visible;
    }
}

#[add_observer(plugin = MenusPlugin)]
fn hide_menus(
    deactivate: On<DeactivateMenu>,
    mut menus: Query<&mut Visibility, With<MenuHidesWhenClosed>>,
) {
    if let Ok(mut visibility) = menus.get_mut(deactivate.menu) {
        *visibility = Visibility::Hidden;
    }
}

#[add_observer(plugin = MenusPlugin)]
fn despawn_menus(
    deactivate: On<DeactivateMenu>,
    mut menus: Query<Entity, With<MenuDespawnsWhenClosed>>,
    mut commands: Commands,
) {
    if let Ok(menu) = menus.get_mut(deactivate.menu) {
        commands.entity(menu).despawn();
    }
}

#[add_system(plugin = MenusPlugin, schedule = PostUpdate, before = activate_stack_current)]
fn remove_despawned_menus(
    mut menu_stack: ResMut<MenuStack>,
    mut commands: Commands,
    entities: Query<()>,
) {
    for menu in menu_stack.stack.clone() {
        if entities.get(menu).is_err() {
            menu_stack.remove(menu);
            commands.trigger(DeactivateMenu { menu });

            if menu_stack.current == Some(menu) {
                menu_stack.current = None;
            }
        }
    }
}
