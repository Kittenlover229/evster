use std::alloc::{alloc, dealloc, Layout};
use std::borrow::Borrow;
use std::{cell::Cell, ptr::NonNull, rc::Rc};

use crate::Position;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ActorTemplate {
    display_name: String,
    resource_name: String,
}

impl ActorTemplate {
    pub fn new(name: impl ToString, resource_name: impl ToString) -> Self {
        Self {
            display_name: name.to_string(),
            resource_name: resource_name.to_string(),
        }
    }

    pub fn resource_name(&self) -> &str {
        &self.resource_name
    }

    pub fn name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Actor {
    template: Rc<ActorTemplate>,
}

impl Actor {
    pub fn template(&self) -> &ActorTemplate {
        &self.template
    }
}

impl<'a> From<Rc<ActorTemplate>> for Actor {
    fn from(template: Rc<ActorTemplate>) -> Self {
        Self { template }
    }
}

struct WorldActor {
    pub cached_position: Position,
}

struct ActorHeapInstance {
    // Data that i always valid when referencing the actor
    pub(crate) weak_keep_alive: (Cell<usize>, Actor),

    // Data that is only valid while the actor exists in the world
    pub(crate) valid_actor_data: Option<WorldActor>,
}

impl ActorHeapInstance {
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }
}

// An owning handle to an existing actor
pub struct ActorHandle {
    heap: NonNull<ActorHeapInstance>,
}

impl std::fmt::Debug for ActorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorHandle")
            .field("heap", &self.heap)
            .finish()
    }
}

impl ActorHandle {
    pub fn from_actor(actor: Actor, cached_position: Position) -> Self {
        let heap = unsafe {
            let heap = alloc(ActorHeapInstance::layout()) as *mut ActorHeapInstance;
            heap.as_uninit_mut().unwrap().write(ActorHeapInstance {
                weak_keep_alive: (Cell::new(1), actor),
                valid_actor_data: Some(WorldActor { cached_position }),
            });
            heap
        };

        Self {
            heap: NonNull::new(heap).unwrap(),
        }
    }
}

impl Drop for ActorHandle {
    fn drop(&mut self) {
        unsafe {
            // Invalidate the actor data
            self.heap.as_mut().valid_actor_data = None;
        }
    }
}

impl AsRef<Actor> for ActorHandle {
    fn as_ref(&self) -> &Actor {
        unsafe { &self.heap.as_ref().weak_keep_alive.1 }
    }
}

impl Borrow<Actor> for ActorHandle {
    fn borrow(&self) -> &Actor {
        unsafe { &self.heap.as_ref().weak_keep_alive.1 }
    }
}

impl ActorHandle {
    fn drop(&mut self) {
        let heap = unsafe { self.heap.as_mut() };

        // Invalidate the actor data
        heap.valid_actor_data = None;
        let (refs, _) = &heap.weak_keep_alive;
        refs.set(refs.get() - 1);
        if refs.get() == 0 {
            unsafe { dealloc(self.heap.as_ptr() as *mut u8, ActorHeapInstance::layout()) };
        }
    }
}

pub struct ActorReference {
    heap: NonNull<ActorHeapInstance>,
}
