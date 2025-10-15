use std::marker::PhantomData;

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
pub use bevy_pretty_nice_input_derive::Action;

#[derive(EntityEvent)]
pub struct JustPressed<T: Action> {
    #[event_target]
    pub input: Entity,
    pub action: T,
    pub data: ActionData,
}

#[derive(EntityEvent)]
pub struct Pressed<T: Action> {
    #[event_target]
    pub input: Entity,
    pub action: T,
    pub data: ActionData,
}

#[derive(EntityEvent)]
pub struct JustReleased<T: Action> {
    #[event_target]
    pub input: Entity,
    pub action: T,
    pub data: ActionData,
}

pub enum AxisDirection {
    X,
    Y,
}

pub enum MouseScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait Binding {}

#[derive(Component)]
pub enum Binding1D {
    Key(KeyCode),
    KeyAxis(KeyCode, KeyCode),
    GamepadAxis(GamepadAxis),
    MouseButton(MouseButton),
    MouseMove(AxisDirection),
    MouseScroll(MouseScrollDirection),
    MouseScrollAxis(AxisDirection),
}
impl Binding for Binding1D {}

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

#[derive(Component)]
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
impl Binding for Binding2D {}

#[derive(Component)]
pub struct Binding3D {
    pub x: Binding1D,
    pub y: Binding1D,
    pub z: Binding1D,
}
impl Binding for Binding3D {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionData {
    Axis1D(f32),
    Axis2D(Vec2),
    Axis3D(Vec3),
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
}

pub trait Action: Send + Sync + 'static {}

#[macro_export]
macro_rules! input {
    ( $action:ty, [$( $binding:expr ),* $(,)?], [$( $condition:expr ),* $(,)?]$(,)? ) => {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Actions<$action>[
			::bevy::prelude::related!(::bevy_pretty_nice_input::Bindings[
				$( $binding )*
			]),
			$( $condition ),*
		])
    };
    ( $action:ty, [$( $binding:expr ),* $(,)?]$(,)? ) => {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Actions<$action>[
			::bevy::prelude::related!(::bevy_pretty_nice_input::Bindings[
				$( $binding )*
			])
		])
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

#[derive(Component)]
#[relationship_target(relationship = ActionOf<T>)]
pub struct Actions<T: Action>(#[relationship] Vec<Entity>, PhantomData<T>);

#[derive(Component)]
#[relationship(relationship_target = Actions<T>)]
pub struct ActionOf<T: Action>(#[relationship] Entity, PhantomData<T>);

#[derive(Component)]
#[relationship_target(relationship = BindingOf)]
pub struct Bindings(#[relationship] Vec<Entity>);

#[derive(Component)]
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
