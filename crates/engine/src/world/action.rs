use crate::{ActorReference, AsPosition, Position};

#[derive(Debug, Clone, Hash, PartialEq)]
#[non_exhaustive]
pub enum Action {
    MoveActor {
        actor_ref: ActorReference,
        to: Position,
    },
}

impl Action {
    pub fn move_actor(actor: ActorReference, to: impl AsPosition) -> Self {
        Action::MoveActor {
            actor_ref: actor,
            to: to.into(),
        }
    }
}
