use crate::{Position};

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Action {
    MoveActor { from: Position, to: Position },
}
