use std::marker::PhantomData;

use bevy::ecs::query::QueryFilter;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
pub use bevy_pretty_nice_input_derive::Action;

use crate::bundles::observe;

pub mod bundles;

#[macro_export]
macro_rules! input {
    ( $action:ty, [$( $binding:expr ),* $(,)?], [$( $condition:expr ),* $(,)?]$(,)? ) => {(
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Actions<$action>[(
			::bevy::prelude::related!(::bevy_pretty_nice_input::Bindings[$((
				Name::new(format!("Binding of {}", ::bevy::prelude::ShortName::of::<$action>())),
				::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::binding),
				::bevy_pretty_nice_input::BindingParts::spawn($binding),
			)),*]),

			Name::new(format!("Action of {}", ::bevy::prelude::ShortName::of::<$action>())),
			::bevy_pretty_nice_input::PrevActionData::default(),
			::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action::<$action>),
			::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action_2::<$action>),
			::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action_prev_set::<$action>),

			::bevy::prelude::related!(::bevy_pretty_nice_input::Conditions[$((
				Name::new(format!("Condition of {}", ::bevy::prelude::ShortName::of::<$action>())),
				{
					use ::bevy_pretty_nice_input::Condition;
					let condition = $condition;
					(condition.bundle::<$action>(), condition)
				}
			)),*]),
		)]),
    )};

    ( $action:ty, [$( $binding:expr ),* $(,)?]$(,)? ) => {
        $crate::input!($action, [$($binding),*], [])
    };
}

#[macro_export]
macro_rules! input_transition {
    ( $action:ty: $from:ty [<=>] $to:ty, [$( $binding:expr ),* $(,)?], [$( $condition:expr ),* $(,)?]$(,)? ) => {
        ()
    };
}

#[derive(EntityEvent)]
pub struct JustPressed<T: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<T>,
}

impl<T: Action> Clone for JustPressed<T> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}

#[derive(EntityEvent)]
pub struct Pressed<T: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<T>,
}

impl<T: Action> Clone for Pressed<T> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}

#[derive(EntityEvent)]
pub struct JustReleased<T: Action> {
    #[event_target]
    pub input: Entity,
    pub _marker: PhantomData<T>,
}

impl<T: Action> Clone for JustReleased<T> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum AxisDirection {
    X,
    Y,
}

impl AxisDirection {
    pub fn index(&self) -> usize {
        match self {
            AxisDirection::X => 0,
            AxisDirection::Y => 1,
        }
    }
}

#[derive(Debug)]
pub enum MouseScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

mod binding_parts {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct Key(pub bevy::prelude::KeyCode);

    #[derive(Component)]
    pub struct KeyAxis(
        pub bevy::prelude::KeyCode,
        pub bevy::prelude::KeyCode,
        pub bool,
        pub bool,
    );

    #[derive(Component)]
    pub struct GamepadAxis(pub bevy::prelude::GamepadAxis);

    #[derive(Component)]
    pub struct MouseButton(pub bevy::prelude::MouseButton);

    #[derive(Component)]
    pub struct MouseMoveAxis(pub crate::AxisDirection);

    #[derive(Component)]
    pub struct MouseScroll(pub crate::MouseScrollDirection);

    #[derive(Component)]
    pub struct MouseScrollAxis(pub crate::AxisDirection);
}

pub mod binding1d {
    use bevy::ecs::spawn::SpawnableList;
    use bevy::prelude::*;

    use crate::{AxisDirection, BindingPartData, BindingPartOf, MouseScrollDirection};

    pub fn key(key: KeyCode) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Key {:?}", key)),
            BindingPartData::default(),
            crate::binding_parts::Key(key),
        ))
    }

    pub fn key_axis(key_pos: KeyCode, key_neg: KeyCode) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Key Axis {:?} / {:?}", key_pos, key_neg)),
            BindingPartData::default(),
            crate::binding_parts::KeyAxis(key_pos, key_neg, false, false),
        ))
    }

    pub fn gamepad_axis(axis: GamepadAxis) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Gamepad Axis {:?}", axis)),
            BindingPartData::default(),
            crate::binding_parts::GamepadAxis(axis),
        ))
    }

    pub fn mouse_button(button: MouseButton) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Mouse Button {:?}", button)),
            BindingPartData::default(),
            crate::binding_parts::MouseButton(button),
        ))
    }

    pub fn mouse_move_axis(axis: AxisDirection) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Mouse Move Axis {:?}", axis)),
            BindingPartData::default(),
            crate::binding_parts::MouseMoveAxis(axis),
        ))
    }

    pub fn mouse_scroll(direction: MouseScrollDirection) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Mouse Scroll {:?}", direction)),
            BindingPartData::default(),
            crate::binding_parts::MouseScroll(direction),
        ))
    }

    pub fn mouse_scroll_axis(axis: AxisDirection) -> impl SpawnableList<BindingPartOf> {
        Spawn((
            Name::new(format!("Mouse Scroll Axis {:?}", axis)),
            BindingPartData::default(),
            crate::binding_parts::MouseScrollAxis(axis),
        ))
    }

    pub fn space() -> impl SpawnableList<BindingPartOf> {
        key(KeyCode::Space)
    }

    pub fn left_shift() -> impl SpawnableList<BindingPartOf> {
        key(KeyCode::ShiftLeft)
    }

    pub fn left_ctrl() -> impl SpawnableList<BindingPartOf> {
        key(KeyCode::ControlLeft)
    }

    pub fn left_click() -> impl SpawnableList<BindingPartOf> {
        mouse_button(MouseButton::Left)
    }

    pub fn right_click() -> impl SpawnableList<BindingPartOf> {
        mouse_button(MouseButton::Right)
    }

    pub fn middle_click() -> impl SpawnableList<BindingPartOf> {
        mouse_button(MouseButton::Middle)
    }

    pub fn scroll_up() -> impl SpawnableList<BindingPartOf> {
        mouse_scroll(MouseScrollDirection::Up)
    }

    pub fn scroll_down() -> impl SpawnableList<BindingPartOf> {
        mouse_scroll(MouseScrollDirection::Down)
    }
}

pub mod binding2d {
    use bevy::ecs::spawn::SpawnableList;
    use bevy::prelude::*;

    use crate::{AxisDirection, BindingPartOf, binding1d::*};

    pub fn wasd() -> impl SpawnableList<BindingPartOf> {
        (
            key_axis(KeyCode::KeyD, KeyCode::KeyA),
            key_axis(KeyCode::KeyW, KeyCode::KeyS),
        )
    }

    pub fn arrow_keys() -> impl SpawnableList<BindingPartOf> {
        (
            key_axis(KeyCode::ArrowRight, KeyCode::ArrowLeft),
            key_axis(KeyCode::ArrowUp, KeyCode::ArrowDown),
        )
    }

    pub fn mouse_move() -> impl SpawnableList<BindingPartOf> {
        (
            mouse_move_axis(AxisDirection::X),
            mouse_move_axis(AxisDirection::Y),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionData {
    Axis1D(f32),
    Axis2D(Vec2),
    Axis3D(Vec3),
}

impl ActionData {
    pub fn x(x: f32) -> Self {
        ActionData::Axis1D(x)
    }

    pub fn xy(x: f32, y: f32) -> Self {
        ActionData::Axis2D(Vec2::new(x, y))
    }

    pub fn xyz(x: f32, y: f32, z: f32) -> Self {
        ActionData::Axis3D(Vec3::new(x, y, z))
    }
}

impl ActionData {
    pub fn as_1d(&self) -> Option<f32> {
        if let ActionData::Axis1D(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_2d(&self) -> Option<Vec2> {
        if let ActionData::Axis2D(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_3d(&self) -> Option<Vec3> {
        if let ActionData::Axis3D(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            ActionData::Axis1D(value) => *value == 0.0,
            ActionData::Axis2D(value) => *value == Vec2::ZERO,
            ActionData::Axis3D(value) => *value == Vec3::ZERO,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct BindingPartData(pub f32);

#[derive(Component, Default, Debug)]
pub struct PrevActionData(pub Option<ActionData>);

pub trait Action: Send + Sync + 'static {}

#[derive(Component)]
pub struct ComponentBuffer<T: Component> {
    timer: Timer,
    marker: PhantomData<T>,
}

#[derive(Component, Debug)]
#[relationship_target(relationship = ActionOf<T>)]
pub struct Actions<T: Action>(#[relationship] Vec<Entity>, PhantomData<T>);

#[derive(Component, Debug)]
#[relationship(relationship_target = Actions<T>)]
pub struct ActionOf<T: Action>(#[relationship] Entity, PhantomData<T>);

#[derive(Component, Debug)]
#[relationship_target(relationship = BindingOf)]
pub struct Bindings(#[relationship] Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = Bindings)]
pub struct BindingOf(#[relationship] Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = BindingPartOf)]
pub struct BindingParts(#[relationship] Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = BindingParts)]
pub struct BindingPartOf(#[relationship] Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = ConditionOf)]
pub struct Conditions(#[relationship] Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = Conditions)]
pub struct ConditionOf(#[relationship] Entity);

#[derive(Component)]
pub struct InputDisabled;

pub trait Condition {
    fn bundle<T: Action>(&self) -> impl Bundle;
}

fn condition_pass(update: On<ConditionedBindingUpdate>, mut commands: Commands) {
    commands.trigger(ConditionedBindingUpdate {
        target: update.entities[update.index + 1],
        input: update.input,
        action: update.action,
        data: update.data,
        entities: update.entities.clone(),
        index: update.index + 1,
    });
}

#[derive(Component)]
pub struct Cooldown {
    pub duration: f32,
}

impl Cooldown {
    pub fn new(duration: f32) -> Self {
        Self { duration }
    }
}

impl Condition for Cooldown {
    fn bundle<T: Action>(&self) -> impl Bundle {
        observe(condition_pass)
    }
}

#[derive(Component)]
pub struct Filter<F: QueryFilter> {
    pub prev_passed: bool,
    _marker: PhantomData<F>,
}

impl<F: QueryFilter> Default for Filter<F> {
    fn default() -> Self {
        Self {
            prev_passed: false,
            _marker: PhantomData,
        }
    }
}

impl<F: QueryFilter + Send + Sync + 'static> Condition for Filter<F> {
    fn bundle<T: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 inputs: Query<(), F>,
                 mut commands: Commands| {
                    if inputs.get(update.input).is_ok() {
                        commands.trigger(ConditionedBindingUpdate {
                            target: update.entities[update.index + 1],
                            input: update.input,
                            action: update.action,
                            data: update.data,
                            entities: update.entities.clone(),
                            index: update.index + 1,
                        });
                    }
                },
            ),
            observe(filter_add_systems::<T, F>),
        )
    }
}

#[derive(Component)]
pub struct ButtonPress {
    pub threshold: f32,
}

impl ButtonPress {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Default for ButtonPress {
    fn default() -> Self {
        Self { threshold: 0.5 }
    }
}

impl Condition for ButtonPress {
    fn bundle<T: Action>(&self) -> impl Bundle {
        observe(condition_pass)
    }
}

#[derive(Component)]
pub struct ButtonRelease {
    pub threshold: f32,
}

impl ButtonRelease {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Default for ButtonRelease {
    fn default() -> Self {
        Self { threshold: 0.5 }
    }
}

impl Condition for ButtonRelease {
    fn bundle<T: Action>(&self) -> impl Bundle {
        observe(condition_pass)
    }
}

#[derive(Component)]
pub struct InputBuffer {
    pub duration: f32,
    timer: Timer,
}

impl InputBuffer {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

impl Condition for InputBuffer {
    fn bundle<T: Action>(&self) -> impl Bundle {
        observe(condition_pass)
    }
}

#[derive(Component)]
pub struct ResetBuffer;

impl Condition for ResetBuffer {
    fn bundle<T: Action>(&self) -> impl Bundle {
        observe(condition_pass)
    }
}

#[derive(Default)]
pub struct PrettyNiceInputPlugin;

impl Plugin for PrettyNiceInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                binding_part_key,
                binding_part_key_axis,
                binding_part_mouse_move,
            ),
        );
    }
}

#[derive(EntityEvent, Debug, Clone)]
pub struct BindingUpdate {
    #[event_target]
    pub action: Entity,
    pub data: ActionData,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct ConditionedBindingUpdate {
    #[event_target]
    pub target: Entity,
    pub input: Entity,
    pub action: Entity,
    pub data: ActionData,
    pub entities: Vec<Entity>,
    pub index: usize,
}

#[derive(EntityEvent, Debug)]
pub struct BindingPartUpdate {
    #[event_target]
    pub action: Entity,
    pub value: f32,
}

fn binding_part_key(
    mut binding_parts: Query<(&binding_parts::Key, &BindingPartOf, &mut BindingPartData)>,
    mut commands: Commands,
    mut key: MessageReader<KeyboardInput>,
) {
    for message in key.read() {
        for (key, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = message.state.is_pressed() as u8 as f32;
            if key.0 == message.key_code && !message.repeat && data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    action: binding_part_of.0,
                    value,
                });
            }
        }
    }
}

fn binding_part_key_axis(
    mut binding_parts: Query<(
        &mut binding_parts::KeyAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut key_axis: MessageReader<KeyboardInput>,
) {
    for message in key_axis.read() {
        for (mut key_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
            if message.repeat {
                continue;
            }

            if key_axis.0 == message.key_code {
                key_axis.2 = message.state.is_pressed();
            } else if key_axis.1 == message.key_code {
                key_axis.3 = message.state.is_pressed();
            } else {
                continue;
            };

            let value = key_axis.2 as u8 as f32 - key_axis.3 as u8 as f32;
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    action: binding_part_of.0,
                    value,
                });
            }
        }
    }
}

fn binding_part_mouse_move(
    mut binding_parts: Query<(
        &binding_parts::MouseMoveAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseMotion>,
) {
    for message in mouse.read() {
        for (mouse_move, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = match mouse_move.0 {
                AxisDirection::X => message.delta.x,
                AxisDirection::Y => message.delta.y,
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    action: binding_part_of.0,
                    value,
                });
            }
        }
    }
}

pub fn binding(
    update: On<BindingPartUpdate>,
    bindings: Query<(&BindingOf, &BindingParts)>,
    binding_parts: Query<&BindingPartData>,
    mut commands: Commands,
) -> Result {
    let (binding_of, binding_parts_rel) = bindings.get(update.action)?;

    let data = if binding_parts_rel.0.len() == 1 {
        ActionData::Axis1D(binding_parts.get(binding_parts_rel.0[0])?.0)
    } else if binding_parts_rel.0.len() == 2 {
        ActionData::Axis2D(Vec2::new(
            binding_parts.get(binding_parts_rel.0[0])?.0,
            binding_parts.get(binding_parts_rel.0[1])?.0,
        ))
    } else if binding_parts_rel.0.len() == 3 {
        ActionData::Axis3D(Vec3::new(
            binding_parts.get(binding_parts_rel.0[0])?.0,
            binding_parts.get(binding_parts_rel.0[1])?.0,
            binding_parts.get(binding_parts_rel.0[2])?.0,
        ))
    } else {
        return Err(BevyError::from(format!(
            "Binding has invalid number of parts: {}",
            binding_parts_rel.0.len()
        )));
    };

    commands.trigger(BindingUpdate {
        action: binding_of.0,
        data,
    });

    Ok(())
}

pub fn action<T: Action>(
    binding_update: On<BindingUpdate>,
    actions: Query<(&ActionOf<T>, &Conditions)>,
    mut commands: Commands,
) -> Result {
    let (action_of, conditions) = actions.get(binding_update.action)?;
    let input = action_of.0;

    let mut entities = conditions.0.clone();
    entities.push(binding_update.action);
    commands.trigger(ConditionedBindingUpdate {
        target: entities[0],
        input,
        action: binding_update.action,
        data: binding_update.data,
        entities,
        index: 0,
    });
    Ok(())
}

pub fn action_2<T: Action>(
    binding_update: On<ConditionedBindingUpdate>,
    actions: Query<&ActionOf<T>>,
    inputs: Query<Has<InputDisabled>>,
    mut commands: Commands,
    mut prev_data: Local<Option<ActionData>>,
) -> Result {
    let action_of = actions.get(binding_update.action)?;
    let input = action_of.0;

    let input_disabled = inputs.get(input)?;
    let data_is_zero = binding_update.data.is_zero() || input_disabled;
    let prev_is_zero = prev_data.as_ref().is_none_or(ActionData::is_zero);

    if !data_is_zero && prev_is_zero {
        commands.trigger(JustPressed::<T> {
            input,
            data: binding_update.data,
            _marker: PhantomData,
        });
    }
    if !data_is_zero {
        commands.trigger(Pressed::<T> {
            input,
            data: binding_update.data,
            _marker: PhantomData,
        });
    }
    if data_is_zero && !prev_is_zero {
        commands.trigger(JustReleased::<T> {
            input,
            _marker: PhantomData,
        });
    }

    *prev_data = Some(binding_update.data);
    Ok(())
}

pub fn action_prev_set<T: Action>(
    binding_update: On<BindingUpdate>,
    mut actions: Query<&mut PrevActionData>,
) -> Result {
    actions.get_mut(binding_update.action)?.0 = Some(binding_update.data);
    Ok(())
}

pub fn filter_add_systems<T: Action, F: QueryFilter + Send + Sync + 'static>(
    _add: On<ConditionedBindingUpdate>,
    mut commands: Commands,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    commands.queue(|world: &mut World| {
        world.schedule_scope(Update, |_world, schedule| {
            schedule.add_systems(action_prev_filter::<T, F>);
        });
    });
    *done = true;
}

fn action_prev_filter<T: Action, F: QueryFilter + Send + Sync + 'static>(
    inputs: Query<(), F>,
    actions: Query<(&PrevActionData, &ActionOf<T>)>,
    mut filters: Query<(&mut Filter<F>, &ConditionOf)>,
    mut commands: Commands,
) -> Result {
    for (mut filter, condition_of) in filters.iter_mut() {
        let Ok((prev_data, action_of)) = actions.get(condition_of.0) else {
            continue;
        };
        let passed = inputs.get(action_of.0).is_ok();
        if let Some(prev_data) = prev_data.0
            && passed
            && !filter.prev_passed
        {
            commands.trigger(BindingUpdate {
                action: condition_of.0,
                data: prev_data,
            });
        }
        filter.prev_passed = passed;
    }
    Ok(())
}
