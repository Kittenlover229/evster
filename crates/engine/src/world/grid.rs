use std::{mem::swap, rc::Rc};

use hashbrown::HashMap;
use nalgebra_glm::{vec2, Vec2};
use puffin_egui::puffin::profile_function;

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

#[non_exhaustive]
pub struct RaycastIterator<'a> {
    pub(crate) max_distance: f32,
    pub(crate) distance_travelled: f32,
    pub(crate) last_sampled_tile: Vec2,

    pub(crate) grid: &'a Grid,
    pub(crate) step: Vec2,
    pub(crate) from: Position,
}

impl<'a> RaycastIterator<'a> {
    pub fn new(from: Position, direction: Vec2, grid: &'a Grid) -> RaycastIterator<'a> {
        let direction = direction.normalize();
        let step = if direction.x.abs() > direction.y.abs() {
            vec2(direction.x.signum(), direction.y / direction.x.abs())
        } else {
            vec2(direction.x / direction.y.abs(), direction.y.signum())
        };

        Self {
            from,
            step,
            grid,
            max_distance: 30.,
            distance_travelled: 0.,
            last_sampled_tile: Vec2::zeros(),
        }
    }
}

impl<'a> Iterator for RaycastIterator<'a> {
    type Item = &'a Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.distance_travelled >= self.max_distance {
            return None;
        }

        let sample_position = self.from
            + Position::new(
                self.last_sampled_tile.x.round() as i32,
                self.last_sampled_tile.y.round() as i32,
            );

        let tile = self.grid.tile_at(sample_position);

        let new_sampled_tile = self.last_sampled_tile + self.step;

        match tile {
            Some(tile) => {
                self.last_sampled_tile = new_sampled_tile;
                Some(tile)
            }
            None => None,
        }
    }
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
        material: MaterialHandle,
    ) -> (Option<Tile>, &Tile) {
        let pos = position.into();
        let displaced = self.grid.insert(
            pos,
            Tile {
                position: pos,
                material,
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

    pub fn los_check(&self, from: impl AsPosition, to: impl AsPosition) -> Option<&Tile> {
        let (from, to): (Position, Position) = (from.into(), to.into());

        if from == to {
            return self.tile_at(from)
        }

        let direction = Vec2::new((to.x - from.x) as f32, (to.y - from.y) as f32).normalize();

        self.ray_cast(from, direction)
            .take_while(|tile| tile.flags().intersects(TileFlags::PASSTHROUGH))
            .last()
    }

    pub fn ray_cast(&self, from: impl AsPosition, direction: Vec2) -> RaycastIterator {
        RaycastIterator::new(from.into(), direction, self)
    }

    pub fn make_tile_bordered_box(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
        fill: MaterialHandle,
        border: MaterialHandle,
    ) {
        let (from, to) = min_max_aabb_from_rect(from, to);

        let width = to.x - from.x;
        let height = to.y - from.y;

        self.make_tile_box(from, to, fill);

        for i in -1..height + 1 {
            let y = i + from.y;
            self.make_tile_at([from.x - 1, y], border.clone());
            self.make_tile_at([from.x + width, y], border.clone());
        }

        for j in 0..width {
            let x = j + from.x;
            self.make_tile_at([x, from.y - 1], border.clone());
            self.make_tile_at([x, from.y + height], border.clone());
        }
    }

    pub fn tile_neumann_neighbours(&self, at: impl AsPosition) -> [(Position, Option<&Tile>); 4] {
        profile_function!();
        let at = at.into();
        [[0, 1], [1, 0], [0, -1], [-1, 0]]
            .map(Position::from)
            .map(|pos| (at + pos, self.grid.get(&(at + pos))))
    }

    pub fn tile_moore_neighbours(&self, at: impl AsPosition) -> [(Position, Option<&Tile>); 8] {
        profile_function!();
        let at = at.into();
        [
            [0, 1],
            [1, 1],
            [1, 0],
            [1, -1],
            [0, -1],
            [-1, -1],
            [-1, 0],
            [-1, 1],
        ]
        .map(Position::from)
        .map(|pos| (at + pos, self.grid.get(&(at + pos))))
    }

    pub fn make_tile_box(
        &mut self,
        from: impl AsPosition,
        to: impl AsPosition,
        fill: MaterialHandle,
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
        const PASSTHROUGH       = 0b000001;
        const SIGHTBLOCKER      = 0b000010;

        const SOLID             = 0b000010;
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub struct Material {
    pub display_name: String,
    pub resource_name: String,
    pub flags: TileFlags,
}

impl Material {
    pub fn new(
        display_name: impl ToString,
        resource_name: impl ToString,
        flags: TileFlags,
    ) -> MaterialHandle {
        Rc::new(Material {
            display_name: display_name.to_string(),
            resource_name: resource_name.to_string(),
            flags,
        })
    }
}

pub type MaterialHandle = Rc<Material>;

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Tile {
    pub position: Position,
    pub material: MaterialHandle,
    pub occupier: Option<ActorHandle>,
}

impl Tile {
    pub fn is_occupied(&self) -> bool {
        self.occupier.is_some()
    }

    pub fn flags(&self) -> TileFlags {
        self.material.flags
    }

    pub fn world_position(&self) -> Vec2 {
        [self.position.x as f32, self.position.y as f32].into()
    }
}
