use crate::models::letter::Letter;
use crate::models::token::{Token, TokenId};
use crate::models::token_relations::{TokenRelation, TokenRelations};
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write;
use std::ops::{Add, AddAssign, Mul};

#[derive(Debug, Clone)]
pub struct Grid {
    letter_by_pos: BTreeMap<XY, Letter>,
    pos_by_letter: BTreeMap<Letter, Vec<XY>>,
    tokens: Vec<PositionedToken>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct XY {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
    Diagonal,
}

#[derive(Debug, Clone, Copy)]
struct PositionedToken {
    token: TokenId,
    base: XY,
    middle: XY,
}

impl Grid {
    /// Create a new grid with a single token in a given direction
    pub fn new(token: &Token, direction: Direction) -> Self {
        let mut grid = Grid {
            letter_by_pos: Default::default(),
            pos_by_letter: Default::default(),
            tokens: vec![],
        };

        grid.insert(token, XY::ORIGIN, direction);

        grid
    }

    pub fn num_letters(&self) -> i32 {
        self.letter_by_pos.len() as i32
    }

    pub fn enumerate_insertions(&self, relations: &TokenRelations, token: &Token) -> Vec<Grid> {
        // Enumerate all insertions that use a valid pivot. A `BTreeSet` is used to deduplicate
        // them, in case a single insertion covers multiple pivots simultaneously
        let mut insertions = BTreeSet::new();

        for (letter_index, letter) in token.text.letters().iter().enumerate() {
            let n = letter_index as i32;

            if let Some(pivots) = self.pos_by_letter.get(letter) {
                for &pivot in pivots {
                    let mut consider_direction = |direction| {
                        let base = match direction {
                            Direction::Horizontal => XY::new(pivot.x - n, pivot.y),
                            Direction::Vertical => XY::new(pivot.x, pivot.y - n),
                            Direction::Diagonal => XY::new(pivot.x - n, pivot.y - n),
                        };
                        insertions.insert((base, direction));
                    };

                    consider_direction(Direction::Horizontal);
                    consider_direction(Direction::Vertical);
                    consider_direction(Direction::Diagonal);
                }
            }
        }

        // Collect the valid insertions
        insertions
            .into_iter()
            .filter_map(|(base, direction)| self.try_inserted(relations, token, base, direction))
            .collect()
    }

    fn try_inserted(
        &self,
        relations: &TokenRelations,
        token: &Token,
        base: XY,
        direction: Direction,
    ) -> Option<Self> {
        // Check relative positioning constraints
        let middle = base + direction.as_xy() * ((token.text.letters().len() + 1) / 2) as i32;
        for existing_token in &self.tokens {
            let is_valid = match relations.get(token.id, existing_token.token) {
                TokenRelation::IsBefore => middle < existing_token.base,
                TokenRelation::IsAfter => base > existing_token.middle,
                TokenRelation::None => true,
            };

            if !is_valid {
                return None;
            }
        }

        // Check letters
        let mut pos = base;
        for &letter in token.text.letters() {
            let prev_letter = self.letter_by_pos.get(&pos).copied();
            if prev_letter != None && prev_letter != Some(letter) {
                return None;
            }
            pos += direction.as_xy();
        }

        // Insert
        let mut inserted = self.clone();
        inserted.insert(token, base, direction);
        Some(inserted)
    }

    fn insert(&mut self, token: &Token, base: XY, direction: Direction) {
        let middle = base + direction.as_xy() * ((token.text.letters().len() + 1) / 2) as i32;
        let mut pos = base;
        for &letter in token.text.letters() {
            self.letter_by_pos.insert(pos, letter);
            self.pos_by_letter.entry(letter).or_default().push(pos);
            pos += direction.as_xy();
        }
        self.tokens.push(PositionedToken {
            token: token.id,
            base,
            middle,
        });
    }
}

impl XY {
    pub const ORIGIN: XY = XY { x: 0, y: 0 };

    fn new(x: i32, y: i32) -> Self {
        XY { x, y }
    }
}

impl Direction {
    fn as_xy(self) -> XY {
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

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.letter_by_pos.is_empty() {
            return Ok(());
        }

        let x_limits = self
            .letter_by_pos
            .keys()
            .map(|xy| xy.x)
            .minmax()
            .into_option()
            .unwrap();
        let y_limits = self
            .letter_by_pos
            .keys()
            .map(|xy| xy.y)
            .minmax()
            .into_option()
            .unwrap();

        for y in y_limits.0..=y_limits.1 {
            for x in x_limits.0..=x_limits.1 {
                match self.letter_by_pos.get(&XY { x, y }) {
                    None => f.write_char(' ')?,
                    Some(letter) => f.write_char(letter.as_char())?,
                }
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}
