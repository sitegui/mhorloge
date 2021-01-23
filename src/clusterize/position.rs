use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position {
    pub i: i16,
    pub j: i16,
}

impl Position {
    pub fn new(i: i16, j: i16) -> Self {
        Position { i, j }
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position::new(self.i + rhs.i, self.j + rhs.j)
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.i - rhs.i, self.j - rhs.j)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.i += rhs.i;
        self.j += rhs.j;
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        self.i -= rhs.i;
        self.j -= rhs.j;
    }
}

impl Mul<i16> for Position {
    type Output = Position;

    fn mul(self, rhs: i16) -> Self::Output {
        Position::new(self.i * rhs, self.j * rhs)
    }
}
