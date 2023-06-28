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
            Action::MoveActor { from, to } => {}
            _ => {}
        }
    }
}
