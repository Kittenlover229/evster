use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    rc::Rc,
};

use thiserror::Error;

use crate::{Actor, AsPosition, Position};

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

    pub fn get_tile_mut(&mut self, position: impl AsPosition) -> Option<&mut Tile> {
        let position = position.into();
        self.grid
            .get_mut((position.x * self.stride as i32 + position.y) as usize)
    }

    pub fn get_tile(&self, position: impl AsPosition) -> Option<&Tile> {
        let position = position.into();
        self.grid
            .get((position.x * self.stride as i32 + position.y) as usize)
    }

    pub fn put_actor(
        &mut self,
        position: impl AsPosition,
        mut actor: Actor,
    ) -> Result<(Option<Rc<RefCell<Actor>>>, Rc<RefCell<Actor>>), GridError> {
        let position = position.into();

        match self.get_tile_mut(position) {
            Some(tile) => {
                actor.cached_position = Some(tile.position);
                let substituted = tile.occupier.replace(Rc::new(RefCell::new(actor)));
                match &tile.occupier {
                    Some(substitutor) => Ok((substituted, substitutor.to_owned())),
                    _ => unreachable!(),
                }
            }

            None => Err(GridError::OutOfBounds),
        }
    }

    pub fn move_actor(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
    ) -> Option<Rc<RefCell<Actor>>> {
        let (from, to) = (from.into(), to.into());

        if from == to {
            return None;
        }

        let occupier = self.remove_actor(from)?;
        occupier.as_ref().borrow_mut().cached_position = Some(to);
        self.get_tile_mut(to)?.occupier.replace(occupier)
    }

    pub fn remove_actor(&mut self, at: impl AsPosition) -> Option<Rc<RefCell<Actor>>> {
        self.get_tile_mut(at.into()).and_then(|tile| {
            let actor = tile.occupier.take();
            if let Some(actor) = &actor {
                actor.as_ref().borrow_mut().cached_position = None;
            }
            actor
        })
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Tile {
    pub position: Position,
    pub occupier: Option<Rc<RefCell<Actor>>>,

    pub is_solid: bool,
}

impl Tile {
    pub fn is_occupied(&self) -> bool {
        self.occupier.is_some()
    }
}
