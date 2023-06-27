use log::trace;

use crate::{Action, Grid, Tile};

pub struct World {
    pub grid: Grid,
}

impl World {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height),
        }
    }

    pub fn submit_action(&mut self, action: Action) {
        match action {
            Action::MoveActor { from, to } => {
                if !self
                    .grid
                    .get_tile(from)
                    .map(Tile::is_occupied)
                    .unwrap_or(false)
                {
                    // Empty starting tile
                    todo!()
                }

                if self
                    .grid
                    .get_tile(to)
                    .map(Tile::is_occupied)
                    .unwrap_or(true)
                {
                    // Non-empty target tile
                    todo!()
                }

                {
                    let actor = self
                        .grid
                        .get_tile(from)
                        .map(|x| &x.occupier)
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .borrow();

                    let actor_name = actor.template().name();

                    trace!("Moved {actor_name} from {from} to {to}");
                }

                self.grid.move_actor(from, to);
            }
            _ => {}
        }
    }
}
