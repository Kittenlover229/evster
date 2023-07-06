use std::mem::swap;

use nalgebra_glm::Vec2;

pub type Position = nalgebra_glm::I32Vec2;
pub trait AsPosition = Into<Position>;

pub fn position(x: impl Into<i32>, y: impl Into<i32>) -> Position {
    Position::new(x.into(), y.into())
}

pub fn pos_to_vec2(position: impl AsPosition) -> Vec2 {
    let position: Position = position.into();
    [position.x as f32, position.y as f32].into()
}

pub fn vec2_to_pos(vec: Vec2) -> Position {
    [vec.x.round() as i32, vec.y.round() as i32].into()
}

pub fn min_max_aabb_from_rect(a: impl AsPosition, b: impl AsPosition) -> (Position, Position) {
    let (mut a, mut b) = (a.into(), b.into());

    if b.x < a.x {
        swap(&mut a.x, &mut b.x);
    }
    if b.y < a.y {
        swap(&mut a.y, &mut b.y);
    }

    (a, b)
}

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Rectangle {
    min: Position,
    max: Position,
}

impl Rectangle {
    pub fn new(a: impl AsPosition, b: impl AsPosition) -> Self {
        let (min, max) = min_max_aabb_from_rect(a, b);
        Self { min, max }
    }

    pub fn min(&self) -> Position {
        self.min
    }

    pub fn max(&self) -> Position {
        self.max
    }

    pub fn centroid(&self) -> Position {
        self.min / 2 + self.max / 2
    }

    pub fn overlaps(&self, rhs: &Rectangle) -> bool {
        let (xmax1, xmax2) = (self.max.x, rhs.max.x);
        let (ymax1, ymax2) = (self.max.y, rhs.max.y);
        let (xmin1, xmin2) = (self.min.x, rhs.min.x);
        let (ymin1, ymin2) = (self.min.y, rhs.min.y);

        xmax1 >= xmin2 && xmax2 >= xmin1 && ymax1 >= ymin2 && ymax2 >= ymin1
    }
}
