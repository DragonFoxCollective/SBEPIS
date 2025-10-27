#![expect(unsafe_code, reason = "Unsafe code is used to improve performance.")]

use std::marker::PhantomData;

use bevy::ecs::bundle::DynamicBundle;
use bevy::ecs::component::{ComponentId, Components, ComponentsRegistrator, StorageType};
use bevy::ecs::system::IntoObserverSystem;
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
        _ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // SAFETY: Empty function body
    }

    #[inline]
    unsafe fn apply_effect(
        ptr: MovingPtr<'_, core::mem::MaybeUninit<Self>>,
        entity: &mut EntityWorldMut,
    ) {
        // SAFETY: `get_components` does nothing, value was not moved.
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
