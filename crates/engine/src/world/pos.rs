pub type Position = nalgebra_glm::I32Vec2;

pub trait AsPosition: Into<Position> {}

impl<T> AsPosition for T where T: Into<Position> {}
