use crate::models::token::TokenId;
use crate::Token;
use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct PositionedToken {
    token: TokenId,
    start: XY,
    direction: Direction,
    size: i32,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct XY {
    pub y: i32,
    pub x: i32,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
    Diagonal,
}

impl PositionedToken {
    pub fn new(token: &Token, start: XY, direction: Direction) -> Self {
        PositionedToken {
            token: token.id,
            start,
            direction,
            size: token.text.letters().len() as i32,
        }
    }

    pub fn token_id(&self) -> TokenId {
        self.token
    }

    pub fn start(&self) -> XY {
        self.start
    }

    pub fn middle(&self) -> XY {
        self.start + self.direction.as_xy() * (self.size / 2)
    }

    pub fn end(&self) -> XY {
        self.start + self.direction.as_xy() * (self.size - 1)
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn positions(self) -> impl Iterator<Item = XY> {
        (0..self.size).map(move |i| self.start + self.direction.as_xy() * i)
    }
}

impl XY {
    pub const ORIGIN: XY = XY { x: 0, y: 0 };

    pub fn new(x: i32, y: i32) -> Self {
        XY { x, y }
    }
}

impl Direction {
    pub fn as_xy(self) -> XY {
        match self {
            Direction::Horizontal => XY::new(1, 0),
            Direction::Vertical => XY::new(0, 1),
            Direction::Diagonal => XY::new(1, 1),
        }
    }
}

impl Add for XY {
    type Output = XY;

    fn add(self, rhs: Self) -> Self::Output {
        XY::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for XY {
    type Output = XY;

    fn sub(self, rhs: Self) -> Self::Output {
        XY::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl AddAssign for XY {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<i32> for XY {
    type Output = XY;

    fn mul(self, rhs: i32) -> Self::Output {
        XY::new(self.x * rhs, self.y * rhs)
    }
}

impl Add<XY> for PositionedToken {
    type Output = PositionedToken;

    fn add(self, rhs: XY) -> Self::Output {
        PositionedToken {
            token: self.token,
            start: self.start + rhs,
            direction: self.direction,
            size: self.size,
        }
    }
}

impl Sub<XY> for PositionedToken {
    type Output = PositionedToken;

    fn sub(self, rhs: XY) -> Self::Output {
        PositionedToken {
            token: self.token,
            start: self.start - rhs,
            direction: self.direction,
            size: self.size,
        }
    }
}
