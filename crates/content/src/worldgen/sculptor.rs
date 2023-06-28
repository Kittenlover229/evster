use engine::{AsPosition, Grid, Position, TileDescriptor};

pub trait Sculptor {
    fn sculpt_all(&mut self, grid: &mut Grid) {
        self.sculpt(<[i32; 2] as Into<Position>>::into([0, 0]), grid.size, grid);
    }

    fn sculpt(&mut self, from: impl AsPosition, to: impl AsPosition, grid: &mut Grid);
}

impl<F> Sculptor for F
where
    F: FnMut(&mut Grid, Position, Position),
{
    fn sculpt(&mut self, from: impl AsPosition, to: impl AsPosition, grid: &mut Grid) {
        self(grid, from.into(), to.into())
    }
}

pub fn fill_sculptor(fill_with: TileDescriptor) -> impl Sculptor + Sized {
    let fill_with = fill_with.clone();
    move |grid: &mut Grid, from: Position, to: Position| {
        let width = to.x - from.x;
        let height = to.y - from.y;

        for i in 0..height {
            for j in 0..width {
                let y = i + from.y;
                let x = j + from.x;

                grid.make_tile_at([x, y], fill_with.clone());
            }
        }
    }
}

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
