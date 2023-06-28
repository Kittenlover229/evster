use std::rc::Rc;

use hashbrown::HashMap;

use crate::{Actor, ActorHandle, ActorReference, AsPosition, Position};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Grid {
    pub size: Position,
    pub grid: hashbrown::HashMap<Position, Tile>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let mut grid = HashMap::default();
        grid.reserve((width * height).into());

        Self {
            size: [width as i32, height as i32].into(),
            grid,
        }
    }

    pub fn tile_at_mut(&mut self, position: impl AsPosition) -> Option<&mut Tile> {
        self.grid.get_mut(&position.into())
    }

    pub fn tile_at(&self, position: impl AsPosition) -> Option<&Tile> {
        self.grid.get(&position.into())
    }

    pub fn make_tile_at(
        &mut self,
        position: impl AsPosition,
        descriptor: TileDescriptor,
    ) -> (Option<Tile>, &Tile) {
        let pos = position.into();
        let displaced = self.grid.insert(
            pos,
            Tile {
                position: pos,
                descriptor,
                occupier: None,
            },
        );
        (
            displaced,
            self.grid
                .get(&pos)
                .expect("Couldn't get Tile that was just put"),
        )
    }

    pub fn move_actor(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
    ) -> Option<(Option<ActorReference>, ActorReference)> {
        let mut actor = self
            .tile_at_mut(from)
            .map(|x| x.occupier.take())
            .flatten()?;
        let to = to.into();

        let destination = self.tile_at_mut(to).map(|x| &mut x.occupier)?;
        let mover = actor.as_weak();

        assert!(actor.get_data().is_valid());
        actor
            .get_data_mut()
            .valid_actor_data
            .as_mut()?
            .cached_position = to;

        let moved = destination
            .replace(actor)
            .as_ref()
            .map(ActorHandle::as_weak);

        Some((moved, mover))
    }

    pub fn put_actor(&mut self, position: impl AsPosition, actor: Actor) -> Option<ActorReference> {
        let position = position.into();

        match self.tile_at_mut(position) {
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

#[non_exhaustive]
#[derive(Debug)]
pub struct TileDescription {
    display_name: String,
    resource_name: String,
}

impl TileDescription {
    pub fn new(display_name: impl ToString, resource_name: impl ToString) -> TileDescriptor {
        Rc::new(TileDescription {
            display_name: display_name.to_string(),
            resource_name: resource_name.to_string(),
        })
    }
}

pub type TileDescriptor = Rc<TileDescription>;

#[derive(Debug)]
#[non_exhaustive]
pub struct Tile {
    pub position: Position,
    pub descriptor: TileDescriptor,
    pub occupier: Option<ActorHandle>,
}

impl Tile {
    pub fn is_occupied(&self) -> bool {
        self.occupier.is_some()
    }
}
