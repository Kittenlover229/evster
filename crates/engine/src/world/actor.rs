use std::alloc::{alloc, dealloc, Layout};
use std::borrow::Borrow;
use std::{cell::Cell, ptr::NonNull, rc::Rc};

use crate::Position;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Actor {
    template: Rc<ActorTemplate>,
}

impl Actor {
    pub fn template(&self) -> &ActorTemplate {
        &self.template
    }

    pub fn from_template(template: Rc<ActorTemplate>) -> Actor {
        Self { template }
    }
}

impl<'a> From<Rc<ActorTemplate>> for Actor {
    fn from(template: Rc<ActorTemplate>) -> Self {
        Self { template }
    }
}

pub struct ValidActorData {
    pub cached_position: Position,
}

pub struct ActorData {
    // Data that is always valid when referencing the actor
    pub(crate) weak_keep_alive: (Cell<usize>, Actor),

    // Data that is only valid while the actor exists in the world
    pub(crate) valid_actor_data: Option<ValidActorData>,
}

impl ActorData {
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }

    pub fn actor(&self) -> &Actor {
        &self.weak_keep_alive.1
    }

    pub fn is_valid(&self) -> bool {
        self.valid_actor_data.is_some()
    }

    pub fn try_valid_data(&self) -> Option<&ValidActorData> {
        self.valid_actor_data.as_ref()
    }

    pub unsafe fn from_ptr<'a>(ptr: NonNull<ActorData>) -> &'a mut ActorData {
        unsafe { ptr.as_ptr().as_mut().unwrap() }
    }
}

// An owning handle to an existing actor
#[derive(PartialEq, Eq)]
pub struct ActorHandle {
    heap: NonNull<ActorData>,
}

impl std::fmt::Debug for ActorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorHandle")
            .field("actor", self.get_data().actor())
            .finish()
    }
}

impl ActorHandle {
    pub fn from_actor(actor: Actor, cached_position: Position) -> Self {
        let heap = unsafe {
            let heap = alloc(ActorData::layout()) as *mut ActorData;
            heap.as_uninit_mut().unwrap().write(ActorData {
                weak_keep_alive: (Cell::new(1), actor),
                valid_actor_data: Some(ValidActorData { cached_position }),
            });
            heap
        };

        Self {
            heap: NonNull::new(heap).unwrap(),
        }
    }

    pub fn as_weak(&self) -> ActorReference {
        ActorReference::from_heap(self.heap)
    }

    pub fn get_data(&self) -> &ActorData {
        unsafe { ActorData::from_ptr(self.heap) }
    }

    pub fn get_data_mut(&mut self) -> &mut ActorData {
        unsafe { ActorData::from_ptr(self.heap) }
    }

    pub fn valid_data(&self) -> &ValidActorData {
        assert!(self.get_data().is_valid());
        self.get_data().valid_actor_data.as_ref().unwrap()
    }
}

impl Drop for ActorHandle {
    fn drop(&mut self) {
        let heap = unsafe { self.heap.as_mut() };

        // Invalidate the actor data
        heap.valid_actor_data = None;
        let (refs, _) = &heap.weak_keep_alive;
        refs.set(refs.get() - 1);
        if refs.get() == 0 {
            unsafe { dealloc(self.heap.as_ptr() as *mut u8, ActorData::layout()) };
        }
    }
}

impl Borrow<Actor> for ActorHandle {
    fn borrow(&self) -> &Actor {
        unsafe { &self.heap.as_ref().weak_keep_alive.1 }
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct ActorReference {
    heap: NonNull<ActorData>,
}

impl std::fmt::Debug for ActorReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorReference")
            .field("heap", &self.as_actor_ref())
            .finish()
    }
}

impl ActorReference {
    pub(crate) fn from_heap(heap: NonNull<ActorData>) -> Self {
        let (refs, _) = unsafe { &ActorData::from_ptr(heap).weak_keep_alive };
        refs.set(refs.get() + 1);
        Self { heap }
    }

    pub fn as_actor_ref(&self) -> &Actor {
        &unsafe { self.heap.as_ptr().as_mut() }
            .unwrap()
            .weak_keep_alive
            .1
    }

    pub fn try_as_valid(&self) -> Option<(&Actor, &ValidActorData)> {
        unsafe {
            match ActorData::from_ptr(self.heap) {
                ActorData {
                    weak_keep_alive: (_, actor),
                    valid_actor_data: Some(va),
                } => Some((actor, va)),
                _ => None,
            }
        }
    }

    pub fn get_data(&self) -> &ActorData {
        unsafe { ActorData::from_ptr(self.heap) }
    }
}

impl AsRef<Actor> for ActorReference {
    fn as_ref(&self) -> &Actor {
        unsafe { &self.heap.as_ref().weak_keep_alive.1 }
    }
}

impl Clone for ActorReference {
    fn clone(&self) -> Self {
        let (refs, _) = unsafe { &ActorData::from_ptr(self.heap).weak_keep_alive };
        refs.set(refs.get() + 1);
        Self {
            heap: self.heap.clone(),
        }
    }
}

impl Drop for ActorReference {
    fn drop(&mut self) {
        let (refs, _) = unsafe { &ActorData::from_ptr(self.heap).weak_keep_alive };
        refs.set(refs.get() - 1);
        if refs.get() == 0 {
            unsafe { dealloc(self.heap.as_ptr() as *mut u8, ActorData::layout()) };
        }
    }
}
