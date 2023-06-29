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
