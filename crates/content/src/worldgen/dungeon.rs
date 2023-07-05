use engine::{AsPosition, Grid, MaterialHandle, Position};
use nalgebra_glm::{distance2, vec2};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::num::NonZeroU16;

use crate::Sculptor;

struct Room {
    pub(crate) min: Position,
    pub(crate) max: Position,
}

impl Room {
    pub fn centroid(&self) -> Position {
        self.min / 2 + self.max / 2
    }

    pub fn overlaps(&self, rhs: &Room) -> bool {
        let (xmax1, xmax2) = (self.max.x, rhs.max.x);
        let (ymax1, ymax2) = (self.max.y, rhs.max.y);
        let (xmin1, xmin2) = (self.min.x, rhs.min.x);
        let (ymin1, ymin2) = (self.min.y, rhs.min.y);

        xmax1 >= xmin2 && xmax2 >= xmin1 && ymax1 >= ymin2 && ymax2 >= ymin1
    }
}

#[non_exhaustive]
pub struct DungeonSculptor {
    room_amount: NonZeroU16,
    max_trials: u32,

    floor: MaterialHandle,
    wall: MaterialHandle,

    max_room_size: Position,
    min_room_size: Position,

    rng: ThreadRng,
}

impl DungeonSculptor {
    pub fn new(
        room_amount: NonZeroU16,
        room_size: (impl AsPosition, impl AsPosition),
        floor: MaterialHandle,
        wall: MaterialHandle,
    ) -> Self {
        Self {
            max_trials: 0xFFFF,
            min_room_size: room_size.0.into(),
            max_room_size: room_size.1.into(),
            room_amount,
            floor,
            wall,
            rng: thread_rng(),
        }
    }
}

impl Sculptor for DungeonSculptor {
    #[profiling::function]
    fn sculpt(&mut self, from: impl AsPosition, to: impl AsPosition, grid: &mut Grid) {
        let (from, to) = (from.into(), to.into());

        let mut rooms: Vec<Room> = vec![];
        for _ in 0..self.room_amount.into() {
            let new_room = 'try_make_room: loop {
                // HACK: always spawn rooms at odd coordinates so the corridors never merge
                let min_x = self.rng.gen_range(from.x..to.x);
                let min_y = self.rng.gen_range(from.y..to.y);
                let max_x = min_x
                    + self
                        .rng
                        .gen_range(self.min_room_size.x..self.max_room_size.x);
                let max_y = min_y
                    + self
                        .rng
                        .gen_range(self.min_room_size.y..self.max_room_size.y);

                let potential_room = Room {
                    min: Position::new(min_x, min_y),
                    max: Position::new(max_x, max_y),
                };

                for room in &rooms {
                    if room.overlaps(&potential_room) {
                        continue 'try_make_room;
                    }
                }

                break potential_room;
            };

            rooms.push(new_room);
        }
        
        let edges = {
            use delaunator::{triangulate, Point};
            let centroids: Vec<_> = rooms
                .iter()
                .map(|room| room.centroid())
                .map(|c| Point {
                    x: c.x as f64,
                    y: c.y as f64,
                })
                .collect();

            profiling::scope!("Triangulation");
            let mut edges: Vec<_> = vec![];
            let triangles = triangulate(&centroids[..]).triangles;
            for [a, b] in triangles.array_windows::<2>() {
                    let (a, b) = (*a, *b);
                    let ac = rooms[a].centroid();
                    let bc = rooms[b].centroid();
                    let ac = vec2(ac.x as f32, ac.y as f32);
                    let bc = vec2(bc.x as f32, bc.y as f32);

                    edges.push((a, b, distance2(&ac, &bc) as i32))
            }
            edges
        };

        let corridors = {
            profiling::scope!("Kruskal's Algorithm");
            let mut corridors = vec![];
            use pathfinding::undirected::kruskal::kruskal_indices;
            for (from, to, _weight) in kruskal_indices(rooms.len(), &edges[..]) {
                let a = rooms[from].centroid();
                let b = rooms[to].centroid();

                let intersection: Position = if self.rng.gen_bool(0.5) {
                    [a.x, b.y]
                } else {
                    [b.x, a.y]
                }
                .into();

                corridors.push((a, intersection));
                corridors.push((intersection, b));
            }
            corridors
        };

        for (from, to) in corridors {
            grid.make_tile_box(from + Position::new(1, 1), to, self.floor.clone());
            grid.make_tile_at(from, self.floor.clone());
        }

        for room in rooms {
            grid.make_tile_box(room.min, room.max, self.floor.clone());
        }

        // lol
        // TODO: there is a better way, optimize it
        {
            profiling::scope!("Wall Generation");

            let mut walls_to_insert = vec![];
            {
                profiling::scope!("Locating Walls");
                for tile in grid.grid.values() {
                    if tile.material == self.floor {
                        for (pos, neighbour) in grid.tile_moore_neighbours(tile.position) {
                            match neighbour {
                                Some(_) => continue,
                                None => walls_to_insert.push(pos),
                            }
                        }
                    }
                }
            }

            for wall in walls_to_insert {
                grid.make_tile_at(wall, self.wall.clone());
            }
        }
    }
}
