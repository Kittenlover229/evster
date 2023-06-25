use thiserror::Error;

mod actor;
mod pos;

pub use actor::*;
pub use pos::*;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct World {
    pub stride: u16,
    pub grid: Vec<Tile>,
}

#[derive(Error, Debug)]
pub enum GridError {
    #[error("Indexing the world out of bounds!")]
    OutOfBounds,
}

impl World {
    pub fn new(width: u16, height: u16) -> Self {
        let mut grid = vec![];

        for x in 0..width {
            for y in 0..height {
                grid.push(Tile {
                    position: [x, y].map(i32::from).into(),
                    occupier: None,
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

    pub fn put_actor<'a>(
        &'a mut self,
        position: Position,
        actor: Actor,
    ) -> Result<(Option<Actor>, &'a Actor), GridError> {
        match self.get_tile_mut(position) {
            Some(tile) => {
                let substituted = tile.occupier.replace(actor);
                match &tile.occupier {
                    Some(x) => Ok((substituted, x)),
                    _ => unreachable!(),
                }
            }
            None => Err(GridError::OutOfBounds),
        }
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Tile {
    pub position: Position,
    pub occupier: Option<Actor>,
}
