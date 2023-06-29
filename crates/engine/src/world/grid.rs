use std::{mem::swap, rc::Rc};

use hashbrown::HashMap;

use crate::{Actor, ActorHandle, ActorReference, AsPosition, Position};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Grid {
    pub size: Position,
    pub grid: hashbrown::HashMap<Position, Tile>,
}

fn min_max_aabb_from_rect(a: impl AsPosition, b: impl AsPosition) -> (Position, Position) {
    let (mut a, mut b) = (a.into(), b.into());

    if b.x < a.x {
        swap(&mut a.x, &mut b.x);
    }
    if b.y < a.y {
        swap(&mut a.y, &mut b.y);
    }

    (a, b)
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

    pub fn make_tile_bordered_box(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
        fill: TileDescriptor,
        border: TileDescriptor,
    ) {
        let (from, to) = min_max_aabb_from_rect(from, to);

        let width = to.x - from.x;
        let height = to.y - from.y;

        self.make_tile_box(from, to, fill);

        for i in 0..height {
            let y = i + from.y;
            self.make_tile_at([from.x, y], border.clone());
            self.make_tile_at([from.x + width - 1, y], border.clone());
        }

        for j in 1..(width - 1) {
            let x = j + from.x;
            self.make_tile_at([x, from.y], border.clone());
            self.make_tile_at([x, from.y + height - 1], border.clone());
        }
    }

    pub fn make_tile_box(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
        fill: TileDescriptor,
    ) {
        let (from, to) = min_max_aabb_from_rect(from, to);

        let width = to.x - from.x;
        let height = to.y - from.y;

        for i in 0..height {
            for j in 0..width {
                let y = i + from.y;
                let x = j + from.x;

                self.make_tile_at([x, y], fill.clone());
            }
        }
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

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub struct TileFlags: u16 {
        const PASSTHROUGH   = 0b000000;
        const SOLID         = 0b000001;
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct TileDescription {
    pub display_name: String,
    pub resource_name: String,
    pub flags: TileFlags,
}

impl TileDescription {
    pub fn new(
        display_name: impl ToString,
        resource_name: impl ToString,
        flags: TileFlags,
    ) -> TileDescriptor {
        Rc::new(TileDescription {
            display_name: display_name.to_string(),
            resource_name: resource_name.to_string(),
            flags,
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

    pub fn flags(&self) -> TileFlags {
        self.descriptor.flags
    }
}
