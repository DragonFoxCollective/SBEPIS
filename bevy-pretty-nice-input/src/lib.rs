use std::marker::PhantomData;

use bevy::ecs::query::QueryFilter;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
pub use bevy_pretty_nice_input_derive::Action;

#[derive(EntityEvent)]
pub struct JustPressed<T: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<T>,
}

#[derive(EntityEvent)]
pub struct Pressed<T: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<T>,
}

#[derive(EntityEvent)]
pub struct JustReleased<T: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<T>,
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

pub trait Binding {
    fn update_key(&self, key: &KeyboardInput, prev: Option<&ActionData>) -> Option<ActionData>;
    fn update_mouse_move(
        &self,
        mouse: &MouseMotion,
        prev: Option<&ActionData>,
    ) -> Option<ActionData>;
}

#[derive(Component, Debug)]
pub enum Binding1D {
    Key(KeyCode),
    KeyAxis(KeyCode, KeyCode),
    GamepadAxis(GamepadAxis),
    MouseButton(MouseButton),
    MouseMove(AxisDirection),
    MouseScroll(MouseScrollDirection),
    MouseScrollAxis(AxisDirection),
}

impl Binding1D {
    pub fn space() -> Self {
        Self::Key(KeyCode::Space)
    }

    pub fn left_shift() -> Self {
        Self::Key(KeyCode::ShiftLeft)
    }

    pub fn left_ctrl() -> Self {
        Self::Key(KeyCode::ControlLeft)
    }

    pub fn left_click() -> Self {
        Self::MouseButton(MouseButton::Left)
    }

    pub fn right_click() -> Self {
        Self::MouseButton(MouseButton::Right)
    }

    pub fn middle_click() -> Self {
        Self::MouseButton(MouseButton::Middle)
    }

    pub fn scroll_up() -> Self {
        Self::MouseScroll(MouseScrollDirection::Up)
    }

    pub fn scroll_down() -> Self {
        Self::MouseScroll(MouseScrollDirection::Down)
    }
}

impl Binding for Binding1D {
    fn update_key(
        &self,
        keyboard: &KeyboardInput,
        _prev: Option<&ActionData>,
    ) -> Option<ActionData> {
        if let Binding1D::Key(key) = self
            && *key == keyboard.key_code
            && !keyboard.repeat
        {
            Some(ActionData::Axis1D(keyboard.state.is_pressed() as u8 as f32))
        } else {
            None
        }
    }

    fn update_mouse_move(
        &self,
        mouse: &MouseMotion,
        _prev: Option<&ActionData>,
    ) -> Option<ActionData> {
        if let Binding1D::MouseMove(axis) = self {
            let value = match axis {
                AxisDirection::X => mouse.delta.x,
                AxisDirection::Y => mouse.delta.y,
            };
            Some(ActionData::Axis1D(value))
        } else {
            None
        }
    }
}

#[derive(Component, Debug)]
pub struct Binding2D {
    pub x: Binding1D,
    pub y: Binding1D,
}

impl Binding2D {
    pub fn wasd() -> Self {
        Self {
            x: Binding1D::KeyAxis(KeyCode::KeyD, KeyCode::KeyA),
            y: Binding1D::KeyAxis(KeyCode::KeyW, KeyCode::KeyS),
        }
    }

    pub fn arrow_keys() -> Self {
        Self {
            x: Binding1D::KeyAxis(KeyCode::ArrowRight, KeyCode::ArrowLeft),
            y: Binding1D::KeyAxis(KeyCode::ArrowUp, KeyCode::ArrowDown),
        }
    }

    pub fn mouse_move() -> Self {
        Self {
            x: Binding1D::MouseMove(AxisDirection::X),
            y: Binding1D::MouseMove(AxisDirection::Y),
        }
    }
}

macro_rules! impl_binding_update {
    ($update: ident, $message: ident, $as_fn: ident, $( $axis: ident ),+) => {
        paste::paste! {
            fn $update(&self, input: &$message, prev: Option<&ActionData>) -> Option<ActionData> {
                $(let [<prev_ $axis>] = prev.and_then(ActionData::[<$as_fn _ $axis>]);)+

                $(let $axis = self.$axis.$update(input, [<prev_ $axis>].as_ref());)+

                if $($axis.is_none()) &&+ {
                    return None;
                };

                $(let $axis = $axis
                    .or([<prev_ $axis>])
                    .as_ref()
                    .and_then(ActionData::as_1d)
                    .unwrap_or_default();)+

                Some(ActionData::[<$($axis)+>]($($axis),+))
            }
        }
    };
}

impl Binding for Binding2D {
    impl_binding_update!(update_key, KeyboardInput, as_2d, x, y);
    impl_binding_update!(update_mouse_move, MouseMotion, as_2d, x, y);
}

#[derive(Component, Debug)]
pub struct Binding3D {
    pub x: Binding1D,
    pub y: Binding1D,
    pub z: Binding1D,
}

impl Binding for Binding3D {
    impl_binding_update!(update_key, KeyboardInput, as_3d, x, y, z);
    impl_binding_update!(update_mouse_move, MouseMotion, as_3d, x, y, z);
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

    pub fn as_1d_x(&self) -> Option<ActionData> {
        if let ActionData::Axis1D(value) = self {
            Some(ActionData::Axis1D(*value))
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

    pub fn as_2d_x(&self) -> Option<ActionData> {
        if let ActionData::Axis2D(value) = self {
            Some(ActionData::Axis1D(value.x))
        } else {
            None
        }
    }

    pub fn as_2d_y(&self) -> Option<ActionData> {
        if let ActionData::Axis2D(value) = self {
            Some(ActionData::Axis1D(value.y))
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

    pub fn as_3d_x(&self) -> Option<ActionData> {
        if let ActionData::Axis3D(value) = self {
            Some(ActionData::Axis1D(value.x))
        } else {
            None
        }
    }

    pub fn as_3d_y(&self) -> Option<ActionData> {
        if let ActionData::Axis3D(value) = self {
            Some(ActionData::Axis1D(value.y))
        } else {
            None
        }
    }

    pub fn as_3d_z(&self) -> Option<ActionData> {
        if let ActionData::Axis3D(value) = self {
            Some(ActionData::Axis1D(value.z))
        } else {
            None
        }
    }
}

impl ActionData {
    pub fn is_zero(&self) -> bool {
        match self {
            ActionData::Axis1D(value) => *value == 0.0,
            ActionData::Axis2D(value) => *value == Vec2::ZERO,
            ActionData::Axis3D(value) => *value == Vec3::ZERO,
        }
    }
}

#[derive(Component, Debug)]
pub struct PrevActionData(pub Option<ActionData>);

pub trait Action: Send + Sync + 'static {}

#[macro_export]
macro_rules! input {
    ( $action:ty, [$( $binding:expr ),* $(,)?], [$( $condition:expr ),* $(,)?]$(,)? ) => {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Actions<$action>[(
			::bevy::prelude::related!(::bevy_pretty_nice_input::Bindings[$((
				Name::new(format!("Binding of {}", ::bevy::prelude::ShortName::of::<$action>())),
				::bevy_pretty_nice_input::PrevActionData(None),
				$binding
			))*]),

			Name::new(format!("Action of {}", ::bevy::prelude::ShortName::of::<$action>())),
			::bevy::ui_widgets::observe(::bevy_pretty_nice_input::action::<$action>),
			$( $condition ),*
		)])
    };
    ( $action:ty, [$( $binding:expr ),* $(,)?]$(,)? ) => {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Actions<$action>[(
			::bevy::prelude::related!(::bevy_pretty_nice_input::Bindings[$((
				Name::new(format!("Binding of {}", ::bevy::prelude::ShortName::of::<$action>())),
				::bevy_pretty_nice_input::PrevActionData(None),
				$binding
			))*]),

			Name::new(format!("Action of {}", ::bevy::prelude::ShortName::of::<$action>())),
			::bevy::ui_widgets::observe(::bevy_pretty_nice_input::action::<$action>),
		)])
    };
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

#[derive(Component)]
pub struct ComponentBuffer<T: Component> {
    timer: Timer,
    marker: std::marker::PhantomData<T>,
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

#[derive(Component)]
pub struct InputDisabled;

#[derive(Component)]
pub struct Filter<T: QueryFilter>(PhantomData<T>);

impl<T: QueryFilter> Filter<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: QueryFilter> Default for Filter<T> {
    fn default() -> Self {
        Self(PhantomData)
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

#[derive(Component)]
pub struct ResetBuffer;

#[derive(Default)]
pub struct PrettyNiceInputPlugin;

impl Plugin for PrettyNiceInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                binding::<Binding1D>,
                binding::<Binding2D>,
                binding::<Binding3D>,
            ),
        );
    }
}

#[derive(EntityEvent, Debug)]
pub struct BindingUpdate {
    // this thing is kinda upside down, deal with it
    #[event_target]
    pub action: Entity,
    pub data: ActionData,
}

fn binding<T: Binding + Component>(
    mut bindings: Query<(&T, &BindingOf, &mut PrevActionData)>,
    mut commands: Commands,
    mut key: MessageReader<KeyboardInput>,
    mut mouse_move: MessageReader<MouseMotion>,
) {
    for message in key.read() {
        for (binding, binding_of, mut prev) in bindings.iter_mut() {
            if let Some(data) = binding.update_key(message, prev.0.as_ref()) {
                // TODO: this doesn't quite work for KeyAxis since it would call twice, same for B2D and B3D. yet mousemove needs adding up
                commands.trigger(BindingUpdate {
                    action: binding_of.0,
                    data,
                });
                prev.0 = Some(data);
            }
        }
    }

    for message in mouse_move.read() {
        for (binding, binding_of, mut prev) in bindings.iter_mut() {
            if let Some(data) = binding.update_mouse_move(message, prev.0.as_ref()) {
                commands.trigger(BindingUpdate {
                    action: binding_of.0,
                    data,
                });
                prev.0 = Some(data);
            }
        }
    }
}

pub fn action<T: Action>(
    binding_update: On<BindingUpdate>,
    actions: Query<&ActionOf<T>>,
    mut commands: Commands,
    mut prev_data: Local<Option<ActionData>>,
) -> Result {
    let input = actions.get(binding_update.action)?.0;

    commands.trigger(Pressed::<T> {
        input,
        data: binding_update.data,
        _marker: PhantomData,
    });

    *prev_data = Some(binding_update.data);
    Ok(())
}
