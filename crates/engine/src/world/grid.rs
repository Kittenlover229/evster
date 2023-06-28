use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    rc::Rc,
};

use thiserror::Error;

use crate::{Actor, ActorHandle, AsPosition, Position};

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
    ) -> Option<Option<ActorHandle>> {
        let position = position.into();

        match self.get_tile_mut(position) {
            Some(tile) => Some(
                tile.occupier
                    .replace(ActorHandle::from_actor(actor, position)),
            ),

            None => None,
        }
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Tile {
    pub position: Position,
    pub occupier: Option<ActorHandle>,

    pub is_solid: bool,
}

impl Tile {
    pub fn is_occupied(&self) -> bool {
        self.occupier.is_some()
    }
}
