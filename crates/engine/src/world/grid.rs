use thiserror::Error;

use crate::{Actor, ActorHandle, ActorReference, AsPosition, Position};

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

    pub fn move_actor(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
    ) -> Option<(Option<ActorReference>, ActorReference)> {
        let mut actor = self
            .get_tile_mut(from)
            .map(|x| x.occupier.take())
            .flatten()?;
        let to = to.into();

        let destination = self.get_tile_mut(to).map(|x| &mut x.occupier)?;
        let mover = actor.as_weak();

        assert!(actor.get_data().is_valid());
        actor
            .get_data_mut()
            .valid_actor_data
            .as_mut()
            .unwrap()
            .cached_position = to;

        let moved = destination
            .replace(actor)
            .as_ref()
            .map(ActorHandle::as_weak);

        Some((moved, mover))
    }

    pub fn put_actor(&mut self, position: impl AsPosition, actor: Actor) -> Option<ActorReference> {
        let position = position.into();

        match self.get_tile_mut(position) {
            Some(tile) => {
                let handle = ActorHandle::from_actor(actor, position);
                let weak = handle.as_weak();
                tile.occupier.replace(handle);
                Some(weak)
            }

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
