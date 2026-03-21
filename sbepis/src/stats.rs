use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::prelude::*;

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
#[auto_add_plugin(plugin = SbepisPlugin)]
struct StatsPlugin;

#[derive(Component)]
pub struct Stat<T> {
    base: f32,
    op_multiply_before_total: f32,
    op_add_total: f32,
    op_multiply_after_total: f32,
    _phantom_data: PhantomData<T>,
}

impl<T> Stat<T> {
    pub fn new(base: f32) -> Self {
        Stat {
            base,
            op_multiply_before_total: 0.0,
            op_add_total: 0.0,
            op_multiply_after_total: 0.0,
            _phantom_data: default(),
        }
    }

    pub fn total(&self) -> f32 {
        (self.base * (1.0 + self.op_multiply_before_total) + self.op_add_total)
            * (1.0 + self.op_multiply_after_total)
    }

    fn clear(&mut self) {
        self.op_multiply_before_total = 0.0;
        self.op_add_total = 0.0;
        self.op_multiply_after_total = 0.0;
    }

    fn add(&mut self, modifier: &impl StatModifier<T>) {
        self.op_multiply_before_total += modifier.multiply_before();
        self.op_add_total += modifier.add();
        self.op_multiply_after_total += modifier.multiply_after();
    }
}

pub struct StatHook;

impl<S: Send + Sync + 'static> AutoPluginBuildHook<S> for StatHook {
    fn on_build(&self, app: &mut App) {
        app.add_systems(PreUpdate, clear_stat::<S>);
    }
}

fn clear_stat<S: Send + Sync + 'static>(mut stats: Query<&mut Stat<S>>) {
    for mut stat in stats.iter_mut() {
        stat.clear();
    }
}

pub trait StatModifier<Stat> {
    fn multiply_before(&self) -> f32 {
        0.0
    }

    fn add(&self) -> f32 {
        0.0
    }

    fn multiply_after(&self) -> f32 {
        0.0
    }
}

pub struct StatModifierHook<S>(PhantomData<S>);

impl<S> Default for StatModifierHook<S> {
    fn default() -> Self {
        Self(default())
    }
}

impl<S: Send + Sync + 'static, T: StatModifier<S> + Component + 'static> AutoPluginBuildHook<T>
    for StatModifierHook<S>
{
    fn on_build(&self, app: &mut App) {
        app.add_systems(PreUpdate, add_modifier::<S, T>.after(clear_stat::<S>));
    }
}

fn add_modifier<S: Send + Sync + 'static, T: StatModifier<S> + Component>(
    mut stats: Query<(&T, &mut Stat<S>)>,
) {
    for (modifier, mut stat) in stats.iter_mut() {
        stat.add(modifier);
    }
}

#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpHeight;
#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpStaminaCost;
#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpHoldTime;
