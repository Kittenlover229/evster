use std::rc::Rc;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ActorPrototype {
    name: String,
    sprite_name: String,
}

impl ActorPrototype {
    pub fn new(name: impl ToString, sprite_name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            sprite_name: sprite_name.to_string(),
        }
    }

    pub fn sprite<'a>(&'a self) -> &'a str {
        &self.sprite_name
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

impl Actor {
    pub fn prototype(&self) -> &ActorPrototype {
        &self.proto
    } 
}

impl<'a> From<Rc<ActorPrototype>> for Actor {
    fn from(proto: Rc<ActorPrototype>) -> Self {
        Self { proto }
    }
}
