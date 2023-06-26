use crate::{AsPosition, Position};

#[derive(Debug, Clone, Hash, PartialEq)]
#[non_exhaustive]
pub enum Action {
    MoveActor { from: Position, to: Position },
}

impl Action {
    pub fn move_actor(from: impl AsPosition, to: impl AsPosition) -> Self {
        Action::MoveActor {
            from: from.into(),
            to: to.into(),
        }
    }
}
