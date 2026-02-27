use std::time::{Duration, Instant};

use bevy::ecs::system::{RunSystemError, RunSystemOnce as _};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::SbepisAppPlugin;

pub fn new_test_app<M>(plugins: impl bevy::app::Plugins<M>) -> App {
    use crate::prelude::GameState;

    let mut app = App::new();
    app.add_plugins(SbepisAppPlugin { headless: true })
        .add_plugins(plugins);

    while app.plugins_state() == bevy::app::PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();

    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::InGame);
    app.update();

    info!("Test app ready");

    app
}

pub trait TestAppExt {
    fn run_for(&mut self, secs: f32) -> Result;
    fn mock_key(&mut self, key: KeyCode, state: ButtonState) -> Result;
}

impl TestAppExt for App {
    fn run_for(&mut self, secs: f32) -> Result {
        let end = Instant::now() + Duration::from_secs_f32(secs);

        loop {
            self.update();

            if let Some(exit) = self.should_exit() {
                return Err(BevyError::from(format!(
                    "App exited in test with {:?}",
                    exit
                )));
            };

            if Instant::now() > end {
                return Ok(());
            }
        }
    }

    fn mock_key(&mut self, key: KeyCode, state: ButtonState) -> Result {
        match self.world_mut().run_system_once(
            move |mut keyboard_input: MessageWriter<KeyboardInput>| {
                keyboard_input.write(KeyboardInput {
                    key_code: key,
                    logical_key: Key::Dead(None),
                    state,
                    text: None,
                    repeat: false,
                    window: Entity::PLACEHOLDER,
                });
            },
        ) {
            Ok(()) => Ok(()),
            Err(RunSystemError::Failed(err)) => Err(err),
            Err(RunSystemError::Skipped(_)) => unreachable!(),
        }
    }
}

const EPSILON: f32 = 1.0e-3;
fn near(left: f32, right: f32) -> bool {
    (left - right).abs() < EPSILON
}

pub fn assert_near_f32(left: f32, right: f32) {
    assert!(
        near(left, right),
        "assertion `left ~= right` failed\n  left: {left}\n right: {right}"
    );
}

pub fn assert_near_vec3(left: Vec3, right: Vec3) {
    assert!(
        near(left.x, right.x) && near(left.y, right.y) && near(left.z, right.z),
        "assertion `left ~= right` failed\n  left: {left}\n right: {right}"
    );
}
