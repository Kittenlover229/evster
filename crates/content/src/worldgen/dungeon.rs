use engine::{TileDescriptor, Grid, Position};

use crate::Sculptor;

pub fn bare_dungeon_sculptor(floor: TileDescriptor, wall: TileDescriptor) -> impl Sculptor + Sized {
    let floor = floor.clone();
    let wall = wall.clone();

    move |grid: &mut Grid, from: Position, to: Position| {
        assert!(to.x > from.x);
        assert!(to.y > from.y);

        let width = to.x - from.x;
        let height = to.y - from.y;

        for i in 1..(height - 1) {
            for j in 1..(width - 1) {
                let y = i + from.y;
                let x = j + from.x;

                grid.make_tile_at([x, y], floor.clone());
            }
        }

        for i in 0..height {
            let y = i + from.y;
            grid.make_tile_at([0, y], wall.clone());
            grid.make_tile_at([width - 2, y], wall.clone());
        }

        for j in 1..(width - 1) {
            let x = j + from.x;
            grid.make_tile_at([x, 0], wall.clone());
            grid.make_tile_at([x, height - 1], wall.clone());
        }
    }
}