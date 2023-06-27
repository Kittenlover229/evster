use std::rc::Rc;

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
    pub(crate) cached_position: Option<Position>,
    template: Rc<ActorTemplate>,
}

impl Actor {
    pub fn template(&self) -> &ActorTemplate {
        &self.template
    }

    pub fn position(&self) -> Option<Position> {
        self.cached_position
    }
}

impl<'a> From<Rc<ActorTemplate>> for Actor {
    fn from(template: Rc<ActorTemplate>) -> Self {
        Self {
            template,
            cached_position: None,
        }
    }
}
