use crate::{min_max_aabb_from_rect, pos_to_vec2, vec2_to_pos};

use hashbrown::HashMap;
use nalgebra_glm::{vec2, Vec2};
use puffin_egui::puffin::profile_function;

use crate::{
    Actor, ActorHandle, ActorReference, AsPosition, MaterialFlags, MaterialHandle, Position,
};

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

    pub fn get_tile_mut(&mut self, position: impl AsPosition) -> Option<&mut Tile> {
        self.grid.get_mut(&position.into())
    }

    pub fn get_tile(&self, position: impl AsPosition) -> Option<&Tile> {
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

    pub fn los_check(
        &self,
        from: impl AsPosition,
        to: impl AsPosition,
        max_distance: Option<f32>,
    ) -> bool {
        let (from, to): (Position, Position) = (from.into(), to.into());

        if from == to {
            return true;
        }

        let direction = pos_to_vec2(to - from).normalize();

        let raycast = self.ray_cast(from, direction, max_distance);
        for tile in raycast {
            if tile.position == to {
                return true;
            }

            if tile.is_sight_blocker() {
                return false;
            }
        }

        false
    }

    pub fn ray_cast(
        &self,
        from: impl AsPosition,
        direction: Vec2,
        max_distance: Option<f32>,
    ) -> RaycastIterator {
        RaycastIterator::new(
            from.into(),
            direction,
            max_distance.unwrap_or(std::f32::INFINITY),
            self,
        )
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
        let mut actor = self.get_tile_mut(from).and_then(|x| x.occupier.take())?;
        let to = to.into();

        let destination = self.get_tile_mut(to).map(|x| &mut x.occupier)?;
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

    pub fn is_sight_blocker(&self) -> bool {
        self.material.flags == MaterialFlags::SIGHTBLOCKER
    }

    pub fn is_walkable(&self) -> bool {
        self.material.flags == MaterialFlags::PASSTHROUGH && !self.is_occupied()
    }

    pub fn world_position(&self) -> Vec2 {
        pos_to_vec2(self.position)
    }
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
    pub fn new(
        from: Position,
        direction: Vec2,
        max_distance: f32,
        grid: &'a Grid,
    ) -> RaycastIterator<'a> {
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
            max_distance,
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

        let sample_position = self.from + vec2_to_pos(self.last_sampled_tile);

        let tile = self.grid.get_tile(sample_position);

        let new_sampled_tile = self.last_sampled_tile + self.step;

        match tile {
            Some(tile) => {
                self.last_sampled_tile = new_sampled_tile;
                self.distance_travelled += self.step.magnitude();
                Some(tile)
            }
            None => None,
        }
    }
}
