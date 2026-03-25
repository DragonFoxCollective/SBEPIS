use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use num_traits::One;

use crate::prelude::*;

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
#[auto_add_plugin(plugin = SbepisPlugin)]
struct StatsPlugin;

pub trait StatType {
    type DataType: Default + Send + Sync + 'static;
}

#[derive(Component)]
pub struct Stat<T: StatType> {
    base: T::DataType,
    op_multiply_before_total: T::DataType,
    op_add_total: T::DataType,
    op_multiply_after_total: T::DataType,
    _phantom_data: PhantomData<T>,
}

impl<T: StatType> Stat<T> {
    pub fn new(base: T::DataType) -> Self {
        Stat {
            base,
            op_multiply_before_total: default(),
            op_add_total: default(),
            op_multiply_after_total: default(),
            _phantom_data: default(),
        }
    }

    fn clear(&mut self) {
        self.op_multiply_before_total = default();
        self.op_add_total = default();
        self.op_multiply_after_total = default();
    }
}

impl<T: StatType> Stat<T>
where
    T::DataType:
        Copy + std::ops::Add<Output = T::DataType> + std::ops::Mul<Output = T::DataType> + One,
{
    pub fn total(&self) -> T::DataType {
        (self.base * (T::DataType::one() + self.op_multiply_before_total) + self.op_add_total)
            * (T::DataType::one() + self.op_multiply_after_total)
    }

    fn add(&mut self, modifier: &impl StatModifier<T>) {
        self.op_multiply_before_total = self.op_multiply_before_total + modifier.multiply_before();
        self.op_add_total = self.op_add_total + modifier.add();
        self.op_multiply_after_total = self.op_multiply_after_total + modifier.multiply_after();
    }
}

pub struct StatHook;

impl<T: StatType + Send + Sync + 'static> AutoPluginBuildHook<T> for StatHook {
    fn on_build(&self, app: &mut App) {
        app.add_systems(PreUpdate, clear_stat::<T>);
    }
}

fn clear_stat<T: StatType + Send + Sync + 'static>(mut stats: Query<&mut Stat<T>>) {
    for mut stat in stats.iter_mut() {
        stat.clear();
    }
}

pub trait StatModifier<T: StatType> {
    fn multiply_before(&self) -> T::DataType {
        default()
    }

    fn add(&self) -> T::DataType {
        default()
    }

    fn multiply_after(&self) -> T::DataType {
        default()
    }
}

pub struct StatModifierHook<S>(PhantomData<S>);

impl<S> Default for StatModifierHook<S> {
    fn default() -> Self {
        Self(default())
    }
}

impl<
    T: StatType<
            DataType: Copy
                          + std::ops::Add<Output = T::DataType>
                          + std::ops::Mul<Output = T::DataType>
                          + One,
        > + Send
        + Sync
        + 'static,
    Modifier: StatModifier<T> + Component + 'static,
> AutoPluginBuildHook<Modifier> for StatModifierHook<T>
{
    fn on_build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            add_modifier::<T, Modifier>.after(clear_stat::<T>),
        );
    }
}

fn add_modifier<
    T: StatType<
            DataType: Copy
                          + std::ops::Add<Output = T::DataType>
                          + std::ops::Mul<Output = T::DataType>
                          + One,
        > + Send
        + Sync
        + 'static,
    Modifier: StatModifier<T> + Component,
>(
    mut stats: Query<(&Modifier, &mut Stat<T>)>,
) {
    for (modifier, mut stat) in stats.iter_mut() {
        stat.add(modifier);
    }
}

#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpHeight;
impl StatType for JumpHeight {
    type DataType = f32;
}

#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpStaminaCost;
impl StatType for JumpStaminaCost {
    type DataType = f32;
}

#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpHoldTime;
impl StatType for JumpHoldTime {
    type DataType = f32;
}

/// The max time a grounded player has to jump to activate the next combo
#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpComboMaxTime;
impl StatType for JumpComboMaxTime {
    type DataType = f32;
}

/// The max jump combo level
#[auto_plugin_build_hook(plugin = StatsPlugin, hook = StatHook)]
pub struct JumpComboMaxLevel;
impl StatType for JumpComboMaxLevel {
    type DataType = u32;
}
