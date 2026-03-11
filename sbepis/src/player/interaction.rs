use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_pretty_nice_input::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player::Interact;
use crate::player::camera::PlayerCamera;
use crate::prelude::*;
use crate::util::find_in_ancestors;

pub fn interact_with<T: Component>(
    interact: On<JustPressed<Interact>>,
    bodies: Query<&Player>,
    rapier_context: ReadRapierContext,
    cameras: Query<&GlobalTransform, With<PlayerCamera>>,
    entities: Query<Entity, With<T>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) -> Result {
    let body = bodies.get(interact.input)?;
    let camera = cameras.get(body.camera)?;
    let mut hit_entity: Option<(Option<Entity>, f32)> = None;
    rapier_context.single()?.intersect_ray(
        camera.translation(),
        camera.forward().into(),
        3.0,
        true,
        QueryFilter::default(),
        |entity, intersection| {
            if hit_entity
                .map(|(_, time)| intersection.time_of_impact < time)
                .unwrap_or(true)
                && intersection.time_of_impact > 0.0
            {
                hit_entity = Some((
                    find_in_ancestors(entity, &entities, &parents),
                    intersection.time_of_impact,
                ));
            }
            true
        },
    );

    if let Some((Some(entity), _)) = hit_entity {
        commands.trigger(InteractWith::<T>::new(entity));
    }

    Ok(())
}

#[derive(EntityEvent)]
pub struct InteractWith<T> {
    pub entity: Entity,
    pub _marker: PhantomData<T>,
}
impl<T> InteractWith<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}
