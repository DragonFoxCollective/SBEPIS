#![expect(unsafe_code, reason = "Unsafe code is used to improve performance.")]

use std::marker::PhantomData;

use bevy::ecs::bundle::DynamicBundle;
use bevy::ecs::component::{ComponentId, Components, ComponentsRegistrator, StorageType};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::{IntoObserverSystem, ScheduleSystem};
use bevy::prelude::*;
use bevy::ptr::{MovingPtr, OwningPtr};

/// Helper struct that adds an observer when inserted as a [`Bundle`].
///
/// Stolen from bevy_ui_widgets while it's still experimental.
pub struct AddObserver<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>> {
    observer: I,
    marker: PhantomData<(E, B, M)>,
}

// SAFETY: Empty method bodies.
unsafe impl<
    E: EntityEvent,
    B: Bundle,
    M: Send + Sync + 'static,
    I: IntoObserverSystem<E, B, M> + Send + Sync,
> Bundle for AddObserver<E, B, M, I>
{
    #[inline]
    fn component_ids(_components: &mut ComponentsRegistrator, _ids: &mut impl FnMut(ComponentId)) {
        // SAFETY: Empty function body
    }

    #[inline]
    fn get_component_ids(_components: &Components, _ids: &mut impl FnMut(Option<ComponentId>)) {
        // SAFETY: Empty function body
    }
}

impl<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>> DynamicBundle
    for AddObserver<E, B, M, I>
{
    type Effect = Self;

    #[inline]
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // Forget the pointer so that the value is available in `apply_effect`.
        std::mem::forget(ptr);
    }

    #[inline]
    unsafe fn apply_effect(
        ptr: MovingPtr<'_, core::mem::MaybeUninit<Self>>,
        entity: &mut EntityWorldMut,
    ) {
        let add_observer = unsafe { ptr.assume_init() };
        let add_observer = add_observer.read();
        entity.observe(add_observer.observer);
    }
}

/// Adds an observer as a bundle effect.
pub fn observe<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>>(
    observer: I,
) -> AddObserver<E, B, M, I> {
    AddObserver {
        observer,
        marker: PhantomData,
    }
}

/// Helper struct that adds an [`Update`] system when inserted as a [`Bundle`].
pub struct AddSystems<M, I: IntoScheduleConfigs<ScheduleSystem, M>, S: ScheduleLabel> {
    schedule: S,
    systems: I,
    marker: PhantomData<M>,
}

// SAFETY: Empty method bodies.
unsafe impl<
    M: Send + Sync + 'static,
    I: IntoSystem<(), (), M> + Send + Sync + 'static,
    S: ScheduleLabel,
> Bundle for AddSystems<M, I, S>
{
    #[inline]
    fn component_ids(_components: &mut ComponentsRegistrator, _ids: &mut impl FnMut(ComponentId)) {
        // SAFETY: Empty function body
    }

    #[inline]
    fn get_component_ids(_components: &Components, _ids: &mut impl FnMut(Option<ComponentId>)) {
        // SAFETY: Empty function body
    }
}

impl<M: Send + Sync + 'static, I: IntoSystem<(), (), M>, S: ScheduleLabel> DynamicBundle
    for AddSystems<M, I, S>
{
    type Effect = Self;

    #[inline]
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // Forget the pointer so that the value is available in `apply_effect`.
        std::mem::forget(ptr);
    }

    #[inline]
    unsafe fn apply_effect(
        ptr: MovingPtr<'_, core::mem::MaybeUninit<Self>>,
        entity: &mut EntityWorldMut,
    ) {
        let add_system = unsafe { ptr.assume_init() };
        let add_system = add_system.read();
        entity.world_scope(|world| {
            world.schedule_scope(add_system.schedule, |_world, schedule| {
                schedule.add_systems(add_system.systems);
            })
        });
    }
}

/// Adds an observer as a bundle effect.
pub fn add_systems<M, I: IntoScheduleConfigs<ScheduleSystem, M>, S: ScheduleLabel>(
    schedule: S,
    systems: I,
) -> AddSystems<M, I, S> {
    AddSystems {
        schedule,
        systems,
        marker: PhantomData,
    }
}
