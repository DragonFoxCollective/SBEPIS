use std::marker::PhantomData;

use bevy::ecs::query::QueryFilter;
use bevy::input::gamepad::GamepadAxisChangedEvent;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;
pub use bevy_pretty_nice_input_derive::{Action, input, input_transition};

use crate::bundles::{add_systems, observe};

pub mod bundles;
#[cfg(feature = "debug_graph")]
pub mod debug_graph;

#[derive(EntityEvent)]
pub struct JustPressed<A: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for JustPressed<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}

#[derive(EntityEvent)]
pub struct Pressed<A: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for Pressed<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}

#[derive(EntityEvent)]
pub struct JustReleased<A: Action> {
    #[event_target]
    pub input: Entity,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for JustReleased<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            _marker: PhantomData,
        }
    }
}

#[derive(EntityEvent)]
pub struct Updated<A: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for Updated<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
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

    pub fn zeroed(&self) -> Self {
        match self {
            ActionData::Axis1D(_) => ActionData::Axis1D(0.0),
            ActionData::Axis2D(_) => ActionData::Axis2D(Vec2::ZERO),
            ActionData::Axis3D(_) => ActionData::Axis3D(Vec3::ZERO),
        }
    }

    pub fn length(&self) -> f32 {
        match self {
            ActionData::Axis1D(value) => value.abs(),
            ActionData::Axis2D(value) => value.length(),
            ActionData::Axis3D(value) => value.length(),
        }
    }

    pub fn is_pressed_with(&self, threshold: f32) -> bool {
        self.length() > threshold
    }
}

#[derive(Component, Default, Debug)]
pub struct BindingPartData(pub f32);

#[derive(Component, Debug)]
pub struct PrevActionData(pub ActionData);

#[derive(Component, Default, Debug)]
pub struct PrevAction2Data(pub Option<ActionData>);

pub trait Action: Send + Sync + 'static {
    /// Which filter determines how enabled/disabled input is processed. Generally, this should either be [`IsInputEnabled`] or [`IsInputEnabledInvalidate`].
    type EnableFilter: Condition;
}

/// Gets added when its component is added, and removed after the timer expires when its component is removed.
#[derive(Component)]
pub struct ComponentBuffer<T: Component> {
    timer: Timer,
    _marker: PhantomData<T>,
}

impl<T: Component> ComponentBuffer<T> {
    pub fn observe(duration: f32) -> impl Bundle {
        (
            observe(move |add: On<Add, T>, mut commands: Commands| {
                let mut timer = Timer::from_seconds(duration, TimerMode::Once);
                timer.pause();
                commands.entity(add.entity).insert(ComponentBuffer::<T> {
                    timer,
                    _marker: PhantomData,
                });
            }),
            observe(
                |remove: On<Remove, T>, mut conditions: Query<&mut ComponentBuffer<T>>| -> Result {
                    let mut condition = conditions.get_mut(remove.entity)?;
                    condition.timer.reset();
                    condition.timer.unpause();
                    Ok(())
                },
            ),
            add_systems(PreUpdate, tick_component_buffer::<T>),
        )
    }
}

fn tick_component_buffer<T: Component>(
    mut buffers: Query<(Entity, &mut ComponentBuffer<T>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut buffer) in buffers.iter_mut() {
        buffer.timer.tick(time.delta());
        if buffer.timer.is_finished() {
            commands.entity(entity).remove::<ComponentBuffer<T>>();
        }
    }
}

#[derive(Component, Debug)]
#[relationship_target(relationship = ActionOf<A>)]
pub struct Actions<A: Action>(#[relationship] Vec<Entity>, PhantomData<A>);

#[derive(Component, Debug)]
#[relationship(relationship_target = Actions<A>)]
pub struct ActionOf<A: Action>(#[relationship] Entity, PhantomData<A>);

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
    fn bundle<A: Action>(&self) -> impl Bundle;
}

#[allow(dead_code)]
#[deprecated(note = "TODO marker")]
fn condition_pass(update: On<ConditionedBindingUpdate>, mut commands: Commands) {
    commands.trigger(update.next());
}

pub fn invalidate_pass(invalidate: On<InvalidateData>, mut commands: Commands) {
    commands.trigger(invalidate.next());
}

/// Only lets one valid input pass every duration.
#[derive(Component)]
pub struct Cooldown {
    timer: Timer,
    prev: Option<ConditionedBindingUpdate>,
}

impl Cooldown {
    pub fn new(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.finish();
        Self { timer, prev: None }
    }
}

impl Condition for Cooldown {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut conditions: Query<&mut Cooldown>,
                 mut commands: Commands|
                 -> Result {
                    let mut condition = conditions.get_mut(update.target)?;

                    let data = update.data;
                    let prev_data = condition
                        .prev
                        .replace(update.clone())
                        .map(|prev| prev.data)
                        .unwrap_or(data);

                    if !data.is_zero() && prev_data.is_zero() {
                        if condition.timer.is_finished() {
                            debug!("Cooling down");
                            commands.trigger(update.next());
                            commands.trigger(update.next().with_data(data.zeroed()));
                        } else {
                            debug!("Re-cooling down");
                        }
                        condition.timer.set_mode(TimerMode::Repeating);
                    } else if data.is_zero() {
                        debug!("Un-cooling down");
                        condition.timer.set_mode(TimerMode::Once);
                        commands.trigger(update.next());
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>, mut conditions: Query<&mut Cooldown>| -> Result {
                    let mut condition = conditions.get_mut(invalidate.target)?;
                    condition.prev = None;
                    Ok(())
                },
            ),
        )
    }
}

fn tick_cooldown(mut conditions: Query<&mut Cooldown>, time: Res<Time>, mut commands: Commands) {
    for mut condition in conditions.iter_mut() {
        condition.timer.tick(time.delta());
        if condition.timer.is_finished()
            && condition.timer.mode() == TimerMode::Repeating
            && let Some(prev) = &condition.prev
        {
            debug!("Cooldown finished, sending {:?}", prev.data);
            commands.trigger(prev.next());
            commands.trigger(prev.next().with_data(prev.data.zeroed()));
        }
    }
}

/// Only lets the input pass if the query filter matches.
#[derive(Component)]
pub struct Filter<F: QueryFilter> {
    _marker: PhantomData<F>,
}

pub type FilterBuffered<F> = Filter<With<ComponentBuffer<F>>>;

/// Works best for state machines, when controls can change while the input is disabled.
pub type IsInputEnabled = Filter<Without<InputDisabled>>;

impl<F: QueryFilter> Default for Filter<F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: QueryFilter + Send + Sync + 'static> Condition for Filter<F> {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>, inputs: Query<(), F>, mut commands: Commands| {
                if inputs.get(update.input).is_ok() {
                    commands.trigger(update.next());
                } else {
                    commands.trigger(update.next().with_data(update.data.zeroed()));
                }
            },
        )
    }
}

/// Only lets the input pass if the query filter matches. Otherwise, invalidates the input.
#[derive(Component)]
pub struct InvalidatingFilter<F: QueryFilter> {
    _marker: PhantomData<F>,
}

/// Works best for state-agnostic inputs, like opening/closing menus, where keeping the previous input would be harmful.
pub type IsInputEnabledInvalidate = InvalidatingFilter<Without<InputDisabled>>;

impl<F: QueryFilter> Default for InvalidatingFilter<F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: QueryFilter + Send + Sync + 'static> Condition for InvalidatingFilter<F> {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>, inputs: Query<(), F>, mut commands: Commands| {
                if inputs.get(update.input).is_ok() {
                    debug!(
                        "Filter passed for {} filtering {}",
                        ShortName::of::<A>(),
                        ShortName::of::<F>()
                    );
                    commands.trigger(update.next());
                } else {
                    commands.trigger(InvalidateData::from(&*update).next());
                }
            },
        )
    }
}

/// Rising edge filter.
#[derive(Component)]
pub struct ButtonPress {
    pub threshold: f32,
    prev: Option<ActionData>,
}

impl ButtonPress {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            prev: None,
        }
    }
}

impl Default for ButtonPress {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            prev: None,
        }
    }
}

impl Condition for ButtonPress {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut commands: Commands,
                 mut conditions: Query<&mut ButtonPress>|
                 -> Result {
                    let mut condition = conditions.get_mut(update.target)?;

                    let data = update.data;
                    let prev_data = condition.prev.replace(update.data).unwrap_or(data);

                    if data.is_pressed_with(condition.threshold)
                        && !prev_data.is_pressed_with(condition.threshold)
                    {
                        debug!("Button Pressed");
                        commands.trigger(update.next());
                        commands.trigger(update.next().with_data(data.zeroed()));
                    } else if !data.is_pressed_with(condition.threshold) {
                        debug!("Button Passed");
                        commands.trigger(update.next().with_data(data.zeroed()));
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>,
                 mut conditions: Query<&mut ButtonPress>|
                 -> Result {
                    let mut condition = conditions.get_mut(invalidate.target)?;
                    condition.prev = None;
                    Ok(())
                },
            ),
        )
    }
}

/// Falling edge filter.
#[derive(Component)]
pub struct ButtonRelease {
    pub threshold: f32,
    prev: Option<ActionData>,
}

impl ButtonRelease {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            prev: None,
        }
    }
}

impl Default for ButtonRelease {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            prev: None,
        }
    }
}

impl Condition for ButtonRelease {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut commands: Commands,
                 mut conditions: Query<&mut ButtonRelease>|
                 -> Result {
                    let mut condition = conditions.get_mut(update.target)?;

                    let data = update.data;
                    let prev_data = condition.prev.replace(update.data).unwrap_or(data);

                    if !data.is_pressed_with(condition.threshold)
                        && prev_data.is_pressed_with(condition.threshold)
                    {
                        commands.trigger(update.next().with_data(prev_data));
                        commands.trigger(update.next());
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>,
                 mut conditions: Query<&mut ButtonRelease>|
                 -> Result {
                    let mut condition = conditions.get_mut(invalidate.target)?;
                    condition.prev = None;
                    Ok(())
                },
            ),
        )
    }
}

/// Inverts the update, using the last good update when the current one fails.
#[derive(Component, Default)]
pub struct Invert {
    prev_good: Option<ActionData>,
}

impl Condition for Invert {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             mut conditions: Query<&mut Invert>|
             -> Result {
                let mut condition = conditions.get_mut(update.target)?;

                let data = update.data;
                let prev_good = condition.prev_good;
                if !data.is_zero() {
                    condition.prev_good = Some(data);
                }

                if data.is_zero() {
                    if let Some(prev) = prev_good {
                        commands.trigger(update.next().with_data(prev));
                    } else {
                        // No idea what to do if there's no previous good input. Perhaps a Binding::inverted_default()?
                    }
                } else {
                    commands.trigger(update.next().with_data(data.zeroed()));
                }
                Ok(())
            },
        )
    }
}

/// Continues sending valid updates for a duration after the input stops being valid
#[derive(Component)]
pub struct InputBuffer {
    timer: Timer,
    prev: Option<ConditionedBindingUpdate>,
}

impl InputBuffer {
    pub fn new(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.finish();
        Self { timer, prev: None }
    }

    pub fn force_finish(&mut self) {
        let was_paused = self.timer.is_paused();
        self.timer.unpause();
        self.timer.finish();
        if was_paused {
            self.timer.pause();
        }
    }
}

impl Condition for InputBuffer {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut commands: Commands,
                 mut conditions: Query<&mut InputBuffer>|
                 -> Result {
                    let mut condition = conditions.get_mut(update.target)?;

                    let data = update.data;
                    condition.prev.replace(update.clone());

                    commands.trigger(update.next());
                    if !data.is_zero() {
                        condition.prev = Some(update.clone());
                        condition.timer.reset();
                        condition.timer.pause();
                    } else {
                        condition.timer.unpause();
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>,
                 mut conditions: Query<&mut InputBuffer>|
                 -> Result {
                    let mut condition = conditions.get_mut(invalidate.target)?;
                    condition.prev = None;
                    condition.force_finish();
                    Ok(())
                },
            ),
            observe(
                |reset: On<ResetBufferEvent>,
                 mut commands: Commands,
                 mut condition: Query<&mut InputBuffer>|
                 -> Result {
                    debug!("Resetting input buffer");
                    let mut condition = condition.get_mut(reset.target)?;
                    condition.force_finish();
                    if let Some(prev) = &condition.prev {
                        commands.trigger(prev.next().with_data(prev.data.zeroed()));
                    }
                    Ok(())
                },
            ),
        )
    }
}

fn tick_input_buffer(
    mut conditions: Query<&mut InputBuffer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for mut condition in conditions.iter_mut() {
        condition.timer.tick(time.delta());
        if !condition.timer.is_finished()
            && let Some(prev) = &condition.prev
        {
            debug!("Input Buffer active, sending {:?}", prev.data);
            commands.trigger(prev.next());
        } else if condition.timer.just_finished()
            && let Some(prev) = &condition.prev
        {
            debug!("Input Buffer finished, sending {:?}", prev.data.zeroed());
            commands.trigger(prev.next().with_data(prev.data.zeroed()));
        }
    }
}

#[derive(EntityEvent)]
pub struct ResetBufferEvent {
    #[event_target]
    pub target: Entity,
    pub entities: Vec<Entity>,
    pub index: usize,
}

impl ResetBufferEvent {
    pub fn next(&self) -> Option<Self> {
        self.index.checked_sub(1).map(|index| Self {
            target: self.entities[index],
            entities: self.entities.clone(),
            index,
        })
    }
}

impl From<&ConditionedBindingUpdate> for ResetBufferEvent {
    fn from(update: &ConditionedBindingUpdate) -> Self {
        Self {
            target: update.target,
            entities: update.entities.clone(),
            index: update.index,
        }
    }
}

/// Stops any previous input buffers
#[derive(Component)]
pub struct ResetBuffer;

impl Condition for ResetBuffer {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>, mut commands: Commands| {
                if !update.data.is_zero() {
                    commands.trigger(ResetBufferEvent::from(&*update));
                }
                commands.trigger(update.next());
            },
        )
    }
}

fn pass_reset_buffer(reset: On<ResetBufferEvent>, mut commands: Commands) {
    if let Some(next) = reset.next() {
        commands.trigger(next);
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
                binding_part_gamepad_axis,
                binding_part_mouse_button,
                binding_part_mouse_move,
                binding_part_mouse_scroll,
                binding_part_mouse_scroll_axis,
                tick_cooldown,
                tick_input_buffer,
                action_initialize,
            ),
        )
        .add_observer(pass_reset_buffer);
        #[cfg(feature = "debug_graph")]
        app.init_resource::<debug_graph::DebugGraph>();
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

impl ConditionedBindingUpdate {
    /// Guarunteed when used in conditions, not in the final action event
    pub fn next(&self) -> Self {
        Self {
            target: self.entities[self.index + 1],
            input: self.input,
            action: self.action,
            data: self.data,
            entities: self.entities.clone(),
            index: self.index + 1,
        }
    }

    pub fn with_data(&self, data: ActionData) -> Self {
        Self {
            target: self.target,
            input: self.input,
            action: self.action,
            data,
            entities: self.entities.clone(),
            index: self.index,
        }
    }
}

#[derive(EntityEvent)]
pub struct InvalidateData {
    #[event_target]
    pub target: Entity,
    pub entities: Vec<Entity>,
    pub index: usize,
}

impl InvalidateData {
    /// Guarunteed when used in conditions, not in the final action event
    pub fn next(&self) -> Self {
        Self {
            target: self.entities[self.index + 1],
            entities: self.entities.clone(),
            index: self.index + 1,
        }
    }
}

impl From<&ConditionedBindingUpdate> for InvalidateData {
    fn from(update: &ConditionedBindingUpdate) -> Self {
        Self {
            target: update.target,
            entities: update.entities.clone(),
            index: update.index,
        }
    }
}

#[derive(EntityEvent, Debug)]
pub struct BindingPartUpdate {
    #[event_target]
    pub binding: Entity,
    pub binding_part: Entity,
    pub value: f32,
}

fn binding_part_key(
    mut binding_parts: Query<(
        Entity,
        &binding_parts::Key,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut key: MessageReader<KeyboardInput>,
) {
    for message in key.read() {
        for (entity, key, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = message.state.is_pressed() as u8 as f32;
            if key.0 == message.key_code && !message.repeat && data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

fn binding_part_key_axis(
    mut binding_parts: Query<(
        Entity,
        &mut binding_parts::KeyAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut key_axis: MessageReader<KeyboardInput>,
) {
    for message in key_axis.read() {
        for (entity, mut key_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
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
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

fn binding_part_gamepad_axis(
    mut binding_parts: Query<(
        Entity,
        &binding_parts::GamepadAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut gamepad_axis: MessageReader<GamepadAxisChangedEvent>,
) {
    for message in gamepad_axis.read() {
        for (entity, gamepad_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = message.value;
            if gamepad_axis.0 == message.axis && data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

fn binding_part_mouse_button(
    mut binding_parts: Query<(
        Entity,
        &binding_parts::MouseButton,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut mouse_button: MessageReader<MouseButtonInput>,
) {
    for message in mouse_button.read() {
        for (entity, mouse_button, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = message.state.is_pressed() as u8 as f32;
            if mouse_button.0 == message.button && data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

fn binding_part_mouse_move(
    mut binding_parts: Query<(
        Entity,
        &binding_parts::MouseMoveAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseMotion>,
) {
    for message in mouse.read() {
        for (entity, mouse_move, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = match mouse_move.0 {
                AxisDirection::X => message.delta.x,
                AxisDirection::Y => message.delta.y,
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

fn binding_part_mouse_scroll(
    mut binding_parts: Query<(
        Entity,
        &binding_parts::MouseScroll,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseWheel>,
) {
    for message in mouse.read() {
        for (entity, mouse_scroll, binding_part_of, mut data) in binding_parts.iter_mut() {
            // Doesn't handle unit :/
            let value = match mouse_scroll.0 {
                MouseScrollDirection::Up => message.y.max(0.0),
                MouseScrollDirection::Down => message.y.min(0.0),
                MouseScrollDirection::Left => message.x.max(0.0),
                MouseScrollDirection::Right => message.x.min(0.0),
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
                // Reset to 0 after triggering
                data.0 = 0.0;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value: 0.0,
                });
            }
        }
    }
}

fn binding_part_mouse_scroll_axis(
    mut binding_parts: Query<(
        Entity,
        &binding_parts::MouseScrollAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseWheel>,
) {
    for message in mouse.read() {
        for (entity, mouse_scroll_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
            // Doesn't handle unit :/
            let value = match mouse_scroll_axis.0 {
                AxisDirection::X => message.x,
                AxisDirection::Y => message.y,
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
                // Reset to 0 after triggering
                data.0 = 0.0;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value: 0.0,
                });
            }
        }
    }
}

struct BindingPartUpdateOrData<'a> {
    binding_part_index: usize,
    update_value: f32,
    binding_parts: Box<dyn Fn(Entity) -> Result<f32> + 'a>,
    binding_parts_rel: &'a BindingParts,
}

impl BindingPartUpdateOrData<'_> {
    fn get(&self, index: usize) -> Result<f32> {
        if index == self.binding_part_index {
            Ok(self.update_value)
        } else {
            Ok((self.binding_parts)(self.binding_parts_rel.0[index])?)
        }
    }
}

pub fn binding(
    update: On<BindingPartUpdate>,
    bindings: Query<(&BindingOf, &BindingParts)>,
    binding_parts: Query<&BindingPartData>,
    mut commands: Commands,
) -> Result {
    let (binding_of, binding_parts_rel) = bindings.get(update.binding)?;

    let binding_part_index = binding_parts_rel
        .0
        .iter()
        .position(|&e| e == update.binding_part)
        .ok_or(BevyError::from("Cannot find binding part in binding parts"))?;
    let update_or_data = BindingPartUpdateOrData {
        binding_part_index,
        update_value: update.value,
        binding_parts: Box::new(|entity| Ok(binding_parts.get(entity)?.0)),
        binding_parts_rel,
    };

    let data = if binding_parts_rel.0.len() == 1 {
        ActionData::Axis1D(update_or_data.get(0)?)
    } else if binding_parts_rel.0.len() == 2 {
        ActionData::Axis2D(Vec2::new(update_or_data.get(0)?, update_or_data.get(1)?))
    } else if binding_parts_rel.0.len() == 3 {
        ActionData::Axis3D(Vec3::new(
            update_or_data.get(0)?,
            update_or_data.get(1)?,
            update_or_data.get(2)?,
        ))
    } else {
        return Err(BevyError::from(format!(
            "Binding has invalid number of parts: {}",
            binding_parts_rel.0.len()
        )));
    };

    // debug!("Binding update received {:?}, {:?}", update.value, data);

    commands.trigger(BindingUpdate {
        action: binding_of.0,
        data,
    });

    Ok(())
}

pub fn action<A: Action>(
    binding_update: On<BindingUpdate>,
    mut actions: Query<(&ActionOf<A>, &Conditions, &mut PrevActionData)>,
    mut commands: Commands,
) -> Result {
    // debug!(
    //     "Action update received {} {:?}",
    //     ShortName::of::<A>(),
    //     binding_update.data
    // );

    let (action_of, conditions, mut prev) = actions.get_mut(binding_update.action)?;
    prev.0 = binding_update.data;
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

pub fn action_2<A: Action>(
    update: On<ConditionedBindingUpdate>,
    mut actions: Query<(&ActionOf<A>, &mut PrevAction2Data)>,
    inputs: Query<Has<InputDisabled>>,
    mut commands: Commands,
) -> Result {
    let (action_of, mut prev) = actions.get_mut(update.action)?;
    let input = action_of.0;
    let input_disabled = inputs.get(input)?;

    let data = if input_disabled {
        update.data.zeroed()
    } else {
        update.data
    };
    let prev_data = if let Some(prev_data) = prev.0.replace(data) {
        prev_data
    } else {
        debug!("Initialized {}", ShortName::of::<A>());
        return Ok(());
    };

    if data.as_1d().is_some() {
        debug!(
            "Action 2 update received {} {:?}",
            ShortName::of::<A>(),
            update.data
        );
    }

    if !data.is_zero() && prev_data.is_zero() {
        if data.as_1d().is_some() {
            debug!("Action just pressed {}", ShortName::of::<A>());
        }
        commands.trigger(JustPressed::<A> {
            input,
            data,
            _marker: PhantomData,
        });
    }
    if !data.is_zero() {
        commands.trigger(Pressed::<A> {
            input,
            data,
            _marker: PhantomData,
        });
    }
    commands.trigger(Updated::<A> {
        input,
        data,
        _marker: PhantomData,
    });
    if data.is_zero() && !prev_data.is_zero() {
        if data.as_1d().is_some() {
            debug!("Action just released {}", ShortName::of::<A>());
        }
        commands.trigger(JustReleased::<A> {
            input,
            _marker: PhantomData,
        });
    }

    Ok(())
}

pub fn action_2_invalidate<A: Action>(
    invalidate: On<InvalidateData>,
    mut actions: Query<&mut PrevAction2Data>,
) -> Result {
    let mut prev = actions.get_mut(invalidate.target)?;
    if prev.0.is_some() {
        debug!("Invalidating {}", ShortName::of::<A>());
        prev.0 = None;
    }
    Ok(())
}

fn action_initialize(
    actions: Query<(Entity, &PrevActionData, &PrevAction2Data)>,
    mut commands: Commands,
) -> Result {
    for (entity, prev_data, prev_data_2) in actions.iter() {
        if prev_data_2.0.is_none() {
            commands.trigger(BindingUpdate {
                action: entity,
                data: prev_data.0,
            });
        }
    }
    Ok(())
}

pub fn action_enable<A: Action>(
    remove: On<Remove, InputDisabled>,
    inputs: Query<&Actions<A>>,
    actions: Query<&PrevActionData>,
    mut commands: Commands,
) -> Result {
    for &action in inputs.get(remove.entity)?.0.iter() {
        let prev_data = actions.get(action)?;
        debug!(
            "Enabling input for {} using {:?}",
            ShortName::of::<A>(),
            prev_data.0
        );
        commands.trigger(BindingUpdate {
            action,
            data: prev_data.0,
        });
    }
    Ok(())
}

pub fn transition_on<A: Action, F: Component, T: Component + Default>(
    sprint: On<JustPressed<A>>,
    mut commands: Commands,
) {
    debug!(
        "Transitioning on {} => {}",
        ShortName::of::<F>(),
        ShortName::of::<T>()
    );
    commands
        .entity(sprint.input)
        .remove::<F>()
        .insert(T::default());
}

pub fn transition_off<A: Action, F: Component, T: Component + Default>(
    sprint: On<JustReleased<A>>,
    mut commands: Commands,
) {
    debug!(
        "Transitioning off {} => {}",
        ShortName::of::<F>(),
        ShortName::of::<T>()
    );
    commands
        .entity(sprint.input)
        .remove::<F>()
        .insert(T::default());
}
