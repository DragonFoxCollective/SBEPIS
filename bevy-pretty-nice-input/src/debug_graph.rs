use std::collections::HashSet;

use bevy::ecs::bundle::DynamicBundle;
use bevy::ecs::component::{ComponentId, Components, ComponentsRegistrator, StorageType};
use bevy::prelude::*;
use bevy::ptr::{MovingPtr, OwningPtr};

#[derive(Resource, Default)]
pub struct DebugGraph {
    pub nodes: HashSet<String>,
    pub edges: Vec<(String, String, String)>,
}

pub struct AddGraphEdge {
    from: String,
    to: String,
    edge: String,
}

// SAFETY: Empty method bodies.
unsafe impl Bundle for AddGraphEdge {
    #[inline]
    fn component_ids(_components: &mut ComponentsRegistrator, _ids: &mut impl FnMut(ComponentId)) {
        // SAFETY: Empty function body
    }

    #[inline]
    fn get_component_ids(_components: &Components, _ids: &mut impl FnMut(Option<ComponentId>)) {
        // SAFETY: Empty function body
    }
}

impl DynamicBundle for AddGraphEdge {
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
        let add_graph_edge = unsafe { ptr.assume_init() };
        let add_graph_edge = add_graph_edge.read();
        entity.world_scope(|world| {
            let mut graph = world
                .get_resource_mut::<DebugGraph>()
                .expect("DebugGraph was not added");
            graph.nodes.insert(add_graph_edge.from.clone());
            graph.nodes.insert(add_graph_edge.to.clone());
            graph
                .edges
                .push((add_graph_edge.from, add_graph_edge.to, add_graph_edge.edge));
        })
    }
}

pub fn add_graph_edge<From, To, Edge>() -> AddGraphEdge {
    AddGraphEdge {
        from: ShortName::of::<From>().to_string(),
        to: ShortName::of::<To>().to_string(),
        edge: ShortName::of::<Edge>().to_string(),
    }
}
