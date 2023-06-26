use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use thiserror::Error;

mod actor;
mod pos;
mod world;
mod action;
mod rule;

pub use actor::*;
pub use pos::*;
pub use world::*;
pub use rule::*;
pub use action::*;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Grid {
    pub stride: u16,
    pub grid: Vec<Tile>,
}

#[derive(Error, Debug)]
pub enum GridError {
    #[error("Indexing the world out of bounds!")]
    OutOfBounds,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let mut grid = vec![];

        for x in 0..width {
            for y in 0..height {
                grid.push(Tile {
                    position: [x, y].map(i32::from).into(),
                    occupier: None,
                    is_solid: false,
                })
            }
        }

        Self {
            grid,
            stride: height,
        }
    }

    pub fn get_tile_mut(&mut self, position: Position) -> Option<&mut Tile> {
        self.grid
            .get_mut((position.x * self.stride as i32 + position.y) as usize)
    }

    pub fn get_tile(&self, position: Position) -> Option<&Tile> {
        self.grid
            .get((position.x * self.stride as i32 + position.y) as usize)
    }

    pub fn put_actor<'a, P: AsPosition>(
        &'a mut self,
        position: P,
        actor: Actor,
    ) -> Result<(Option<Rc<RefCell<Actor>>>, Ref<'a, Actor>), GridError> {
        let position = position.into();

        match self.get_tile_mut(position) {
            Some(tile) => {
                let substituted = tile.occupier.replace(Rc::new(RefCell::new(actor)));
                match &tile.occupier {
                    Some(x) => Ok((substituted, x.as_ref().borrow())),
                    _ => unreachable!(),
                }
            }

            None => Err(GridError::OutOfBounds),
        }
    }

    pub fn move_actor<'a, P: AsPosition>(
        &'a mut self,
        from: P,
        to: P,
    ) -> Option<Rc<RefCell<Actor>>> {
        let (from, to) = (from.into(), to.into());

        if from == to {
            return None;
        }

        let occupier = self.remove_actor(from)?;
        self.get_tile_mut(to)?.occupier.replace(occupier)
    }

    pub fn remove_actor<'a, P: AsPosition>(&'a mut self, at: P) -> Option<Rc<RefCell<Actor>>> {
        self.get_tile_mut(at.into())
            .map(|x| x.occupier.take())
            .flatten()
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Tile {
    pub position: Position,
    pub occupier: Option<Rc<RefCell<Actor>>>,

    pub is_solid: bool,
}
