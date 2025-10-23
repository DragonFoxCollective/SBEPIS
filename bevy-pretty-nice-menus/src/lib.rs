use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_pretty_nice_input::{Action, InputDisabled, JustPressed};

#[derive(Default)]
pub struct PrettyNiceMenusPlugin;

impl Plugin for PrettyNiceMenusPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MenuStack::default())
            .add_systems(
                PostUpdate,
                (remove_despawned_menus, activate_stack_current).chain(),
            )
            .add_observer(show_mouse)
            .add_observer(hide_mouse)
            .add_observer(show_menus)
            .add_observer(hide_menus)
            .add_observer(despawn_menus)
            .add_observer(enable_input_managers)
            .add_observer(disable_input_managers)
            .add_observer(disable_input_managers_on_add_menu_with_input)
            .add_observer(disable_input_managers_on_add_menu_input_of);

        app.add_observer(close_menu_on_action::<CloseMenuAction>);
    }
}

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct MenuWithInput;

#[derive(Component)]
pub struct MenuWithMouse;

#[derive(Component)]
pub struct MenuWithoutMouse;

#[derive(Component)]
pub struct MenuHidesWhenClosed;

#[derive(Component)]
pub struct MenuDespawnsWhenClosed;

#[derive(Resource, Default, Debug, Reflect)]
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

#[derive(Action)]
pub struct CloseMenuAction;

/// This is the main sync point for changing the menu stack to activating/deactivating menus.
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

fn show_mouse(
    activate: On<ActivateMenu>,
    menus: Query<(), With<MenuWithMouse>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if menus.get(activate.menu).is_ok()
        && let Ok(mut cursor_options) = cursor_options.single_mut()
    {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

fn hide_mouse(
    activate: On<ActivateMenu>,
    menus: Query<(), With<MenuWithoutMouse>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if menus.get(activate.menu).is_ok()
        && let Ok(mut cursor_options) = cursor_options.single_mut()
    {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}

fn enable_input_managers(
    activate: On<ActivateMenu>,
    menus_with_inputs: Query<(), With<MenuWithInput>>,
    menu_inputs: Query<&MenuInputs>,
    mut commands: Commands,
) {
    if menus_with_inputs.get(activate.menu).is_ok() {
        commands.entity(activate.menu).remove::<InputDisabled>();

        for input in menu_inputs.iter_descendants(activate.menu) {
            commands.entity(input).remove::<InputDisabled>();
        }
    }
}

fn disable_input_managers(
    deactivate: On<DeactivateMenu>,
    menus_with_inputs: Query<(), With<MenuWithInput>>,
    menu_inputs: Query<&MenuInputs>,
    mut commands: Commands,
) {
    if menus_with_inputs.get(deactivate.menu).is_ok() {
        commands.entity(deactivate.menu).insert(InputDisabled);

        for input in menu_inputs.iter_descendants(deactivate.menu) {
            commands.entity(input).insert(InputDisabled);
        }
    }
}

fn disable_input_managers_on_add_menu_with_input(
    add: On<Add, MenuWithInput>,
    mut commands: Commands,
) {
    commands.entity(add.entity).insert(InputDisabled);
}

fn disable_input_managers_on_add_menu_input_of(add: On<Add, MenuInputOf>, mut commands: Commands) {
    commands.entity(add.entity).insert(InputDisabled);
}

#[derive(Component)]
#[relationship_target(relationship = MenuInputOf)]
pub struct MenuInputs(#[relationship] Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = MenuInputs)]
pub struct MenuInputOf(#[relationship] pub Entity);

pub fn close_menu_on_action<Action: bevy_pretty_nice_input::Action>(
    pressed: On<JustPressed<Action>>,
    mut menu_stack: ResMut<MenuStack>,
) {
    menu_stack.remove(pressed.input);
}

pub fn close_menu_on_event<Ev: EntityEvent>(input: On<Ev>, mut menu_stack: ResMut<MenuStack>) {
    menu_stack.remove(input.event_target());
}

pub fn show_menu_on_action<Action: bevy_pretty_nice_input::Action, Menu: Component>(
    _: On<JustPressed<Action>>,
    mut menus: Query<Entity, With<Menu>>,
    mut menu_stack: ResMut<MenuStack>,
) -> Result {
    let menu = menus.single_mut()?;
    menu_stack.push(menu);
    Ok(())
}

fn show_menus(
    activate: On<ActivateMenu>,
    mut menus: Query<&mut Visibility, With<MenuHidesWhenClosed>>,
) {
    if let Ok(mut visibility) = menus.get_mut(activate.menu) {
        *visibility = Visibility::Visible;
    }
}

fn hide_menus(
    deactivate: On<DeactivateMenu>,
    mut menus: Query<&mut Visibility, With<MenuHidesWhenClosed>>,
) {
    if let Ok(mut visibility) = menus.get_mut(deactivate.menu) {
        *visibility = Visibility::Hidden;
    }
}

fn despawn_menus(
    deactivate: On<DeactivateMenu>,
    mut menus: Query<Entity, With<MenuDespawnsWhenClosed>>,
    mut commands: Commands,
) {
    if let Ok(menu) = menus.get_mut(deactivate.menu) {
        commands.entity(menu).despawn();
    }
}

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
