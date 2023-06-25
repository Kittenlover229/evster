use std::rc::Rc;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ActorPrototype {
    name: String,
}

impl ActorPrototype {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Actor {
    proto: Rc<ActorPrototype>,
}

impl<'a> From<Rc<ActorPrototype>> for Actor {
    fn from(proto: Rc<ActorPrototype>) -> Self {
        Self { proto }
    }
}
