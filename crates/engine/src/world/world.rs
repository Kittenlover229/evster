use crate::{Action, ActorReference, Grid, Position, Tile};

pub struct World {
    pub grid: Grid,
}

impl World {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height),
        }
    }

    pub fn submit_action(&mut self, action: Action) -> bool {
        match action {
            Action::MoveActor { actor_ref, to } => self.move_actor(actor_ref, to),
            _ => false,
        }
    }

    fn move_actor(&mut self, actor: ActorReference, destination: Position) -> bool {
        let position = actor.try_as_valid().map(|x| x.1.cached_position);
        let from = match position {
            Some(pos) => pos,
            None => return false,
        };

        if self
            .grid
            .get_tile(destination)
            .map(Tile::is_walkable)
            .map(std::ops::Not::not)
            .unwrap_or(false)
        {
            return false;
        }

        self.grid.move_actor(from, destination);
        true
    }
}
