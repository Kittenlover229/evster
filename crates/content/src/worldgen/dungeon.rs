use engine::{AsPosition, Grid, Position, TileDescriptor};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::num::NonZeroU16;

use crate::Sculptor;

struct Room {
    pub(crate) min: Position,
    pub(crate) max: Position,
}

fn do_intersect(a: &Room, b: &Room) -> bool {
    let (xmax1, xmax2) = (a.max.x, b.max.x);
    let (ymax1, ymax2) = (a.max.y, b.max.y);
    let (xmin1, xmin2) = (a.min.x, b.min.x);
    let (ymin1, ymin2) = (a.min.y, b.min.y);

    return (xmax1 >= xmin2 && xmax2 >= xmin1) && (ymax1 >= ymin2 && ymax2 >= ymin1);
}

#[non_exhaustive]
pub struct DungeonSculptor {
    room_amount: NonZeroU16,
    max_trials: u32,

    floor: TileDescriptor,
    wall: TileDescriptor,

    max_room_size: Position,
    min_room_size: Position,

    rng: ThreadRng,
}

impl DungeonSculptor {
    pub fn new(
        room_amount: NonZeroU16,
        room_size: (impl AsPosition, impl AsPosition),
        floor: TileDescriptor,
        wall: TileDescriptor,
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
    fn sculpt(&mut self, from: impl AsPosition, to: impl AsPosition, grid: &mut Grid) {
        let (from, to) = (from.into(), to.into());
        let width = to.x - from.x;
        let height = to.y - from.y;

        let mut rooms = vec![];
        for room_i in 0..self.room_amount.into() {
            let new_room = 'try_make_room: loop {
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
                    if do_intersect(&room, &potential_room) {
                        continue 'try_make_room;
                    }
                }

                break potential_room;
            };

            rooms.push(new_room);
        }

        for room in rooms {
            grid.make_tile_box(room.min, room.max, self.floor.clone());
        }
    }
}
