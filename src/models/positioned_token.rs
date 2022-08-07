use crate::models::letter::Letter;
use crate::models::token::TokenId;
use crate::Token;
use std::ops::{Add, AddAssign, Mul, Sub};

/// Represent a token with a given [`Direction`]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct OrientedToken {
    token: TokenId,
    direction: Direction,
    size: i16,
}

/// Represent a token positioned in a grid
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PositionedToken {
    start: XY,
    oriented: OrientedToken,
}

/// Represent a given position in the grid
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct XY {
    pub y: i16,
    pub x: i16,
}

/// Represent a possible orientation
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum Direction {
    /// A token with a single letter has no determined direction
    Point,
    Horizontal,
    Vertical,
    Diagonal,
}

impl OrientedToken {
    pub fn orientations(token: &Token, allow_diagonal: bool) -> Vec<Self> {
        let size = token.text.letters().len() as i16;
        let with_direction = |direction| OrientedToken {
            token: token.id,
            direction,
            size,
        };

        match (size, allow_diagonal) {
            (1, _) => vec![with_direction(Direction::Point)],
            (_, false) => vec![
                with_direction(Direction::Horizontal),
                with_direction(Direction::Vertical),
            ],
            (_, true) => vec![
                with_direction(Direction::Horizontal),
                with_direction(Direction::Vertical),
                with_direction(Direction::Diagonal),
            ],
        }
    }

    pub fn token_id(self) -> TokenId {
        self.token
    }

    pub fn direction(self) -> Direction {
        self.direction
    }

    pub fn size(self) -> i16 {
        self.size
    }
}

impl PositionedToken {
    pub fn new(oriented: OrientedToken, start: XY) -> Self {
        PositionedToken { oriented, start }
    }

    pub fn token_id(self) -> TokenId {
        self.oriented.token_id()
    }

    pub fn end(self) -> XY {
        self.start + self.direction().as_xy() * (self.size() - 1)
    }

    pub fn direction(self) -> Direction {
        self.oriented.direction()
    }

    pub fn size(self) -> i16 {
        self.oriented.size()
    }

    /// Iterate over all letters of this positioned token
    pub fn iter(self, token: &Token) -> impl Iterator<Item = (XY, Letter)> + '_ {
        assert_eq!(self.token_id(), token.id);
        token
            .text
            .letters()
            .iter()
            .enumerate()
            .map(move |(i, &letter)| {
                let pos = self.start + self.direction().as_xy() * i as i16;
                (pos, letter)
            })
    }

    /// Iterate over all letters of this positioned token
    pub fn iter_pos(self) -> impl Iterator<Item = XY> {
        (0..self.size()).map(move |i| self.start + self.direction().as_xy() * i)
    }
}

impl XY {
    pub const ORIGIN: XY = XY { x: 0, y: 0 };

    pub fn new(x: i16, y: i16) -> Self {
        XY { x, y }
    }
}

impl Direction {
    pub fn as_xy(self) -> XY {
        match self {
            Direction::Horizontal => XY::new(1, 0),
            Direction::Vertical => XY::new(0, 1),
            Direction::Diagonal => XY::new(1, 1),
            Direction::Point => XY::new(0, 0),
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

impl Mul<i16> for XY {
    type Output = XY;

    fn mul(self, rhs: i16) -> Self::Output {
        XY::new(self.x * rhs, self.y * rhs)
    }
}

impl Add<XY> for PositionedToken {
    type Output = PositionedToken;

    fn add(self, rhs: XY) -> Self::Output {
        PositionedToken {
            start: self.start + rhs,
            oriented: self.oriented,
        }
    }
}

impl Sub<XY> for PositionedToken {
    type Output = PositionedToken;

    fn sub(self, rhs: XY) -> Self::Output {
        PositionedToken {
            start: self.start - rhs,
            oriented: self.oriented,
        }
    }
}
