use crate::models::letter::Letter;
use crate::models::positioned_token::{Direction, PositionedToken, XY};
use crate::models::token::Token;
use crate::models::token_relations::{TokenRelation, TokenRelations};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Grid {
    letter_by_pos: BTreeMap<XY, Letter>,
    pos_by_letter: BTreeMap<Letter, Vec<XY>>,
    tokens: Vec<PositionedToken>,
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

    /// Return the number of determined letters of this grid
    pub fn num_letters(&self) -> i32 {
        self.letter_by_pos.len() as i32
    }

    /// Return the bounding box of this grid
    pub fn space(&self) -> (RangeInclusive<i32>, RangeInclusive<i32>) {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for token in &self.tokens {
            let start = token.start();
            let end = token.end();

            min_x = min_x.min(start.x);
            min_y = min_y.min(start.y);
            max_x = max_x.max(end.x);
            max_y = max_y.max(end.y);
        }

        (min_x..=max_x, min_y..=max_y)
    }

    pub fn size(&self) -> (i32, i32) {
        let (x, y) = self.space();
        let width = x.end() - x.start();
        let height = y.end() - y.start();
        (width, height)
    }

    /// A grid with lower weight is deemed more interesting
    pub fn weight(&self) -> (i32, i32, i32) {
        let (width, height) = self.size();
        let square_side = width.max(height);
        let area = width * height;
        (self.num_letters(), square_side, area)
    }

    /// Return all resulting grids for the valid insertions of the given token
    pub fn enumerate_insertions(&self, relations: &TokenRelations, token: &Token) -> Vec<Grid> {
        // Enumerate all insertions that use a valid pivot. A `BTreeSet` is used to deduplicate
        // them, in case a single insertion covers multiple pivots simultaneously
        let mut insertions = BTreeSet::new();

        for (letter_index, letter) in token.text.letters().iter().enumerate() {
            let n = letter_index as i32;

            if let Some(pivots) = self.pos_by_letter.get(letter) {
                for &pivot in pivots {
                    let mut consider_direction = |direction: Direction| {
                        let start = pivot - direction.as_xy() * n;
                        insertions.insert((start, direction));
                    };

                    if token.text.letters().len() == 1 {
                        consider_direction(Direction::Horizontal);
                    } else {
                        consider_direction(Direction::Horizontal);
                        consider_direction(Direction::Vertical);
                        consider_direction(Direction::Diagonal);
                    }
                }
            }
        }

        // Collect the valid insertions
        insertions
            .into_iter()
            .filter_map(|(start, direction)| self.try_inserted(relations, token, start, direction))
            .collect()
    }

    /// Add a grid to this one, starting it's top left corner at the given position
    pub fn add_grid(&mut self, other: &Grid, start: XY) {
        let (other_x, other_y) = other.space();
        let other_top_left = XY::new(*other_x.start(), *other_y.start());

        for (&other_pos, &letter) in &other.letter_by_pos {
            self.set_letter(letter, other_pos - other_top_left + start);
        }

        self.tokens.extend(
            other
                .tokens
                .iter()
                .map(|&token| token - other_top_left + start),
        );
    }

    pub fn tokens(&self) -> &[PositionedToken] {
        &self.tokens
    }
    
    pub fn get(&self, at: XY) -> Option<Letter> {
        self.letter_by_pos.get(&at).copied()
    }

    fn try_inserted(
        &self,
        relations: &TokenRelations,
        token: &Token,
        start: XY,
        direction: Direction,
    ) -> Option<Self> {
        // Check relative positioning constraints
        for &existing_token in &self.tokens {
            let inserting_token = PositionedToken::new(token, start, direction);
            let is_valid = match relations.get(token.id, existing_token.token_id()) {
                TokenRelation::IsBefore => Self::is_token_after(inserting_token, existing_token),
                TokenRelation::IsAfter => Self::is_token_after(existing_token, inserting_token),
                TokenRelation::None => true,
            };

            if !is_valid {
                return None;
            }
        }

        // Check letters
        let mut pos = start;
        for &letter in token.text.letters() {
            let prev_letter = self.letter_by_pos.get(&pos).copied();
            if prev_letter != None && prev_letter != Some(letter) {
                return None;
            }
            pos += direction.as_xy();
        }

        // Insert
        let mut inserted = self.clone();
        inserted.insert(token, start, direction);
        Some(inserted)
    }

    /// Returns whether a token positioned like `b` is considered "after" another positioned like
    /// `a`
    fn is_token_after(a: PositionedToken, b: PositionedToken) -> bool {
        // `b` must centered after the middle of `a`
        let middle_a = a.middle();
        let middle_b = b.middle();
        let is_readable_as_after = middle_b > middle_a;

        // If they share the same direction, then `b` must be readable as a separate word.
        // That is, `b` must not start in the middle of `a` or immediately after it.
        let end_a = a.end();
        let is_readable_at_the_same_time = match (a.direction(), b.direction()) {
            (Direction::Horizontal, Direction::Horizontal) => {
                a.start().y != b.start().y || b.start().x > end_a.x + 1
            }
            (Direction::Vertical, Direction::Vertical) => {
                a.start().x != b.start().x || b.start().y > end_a.y + 1
            }
            (Direction::Diagonal, Direction::Diagonal) => {
                let line_a = a.start().x - a.start().y;
                let line_b = b.start().x - b.start().y;
                line_a != line_b || b.start().x > end_a.x + 1
            }
            _ => true,
        };

        is_readable_as_after && is_readable_at_the_same_time
    }

    fn insert(&mut self, token: &Token, start: XY, direction: Direction) {
        let mut pos = start;
        for &letter in token.text.letters() {
            self.set_letter(letter, pos);
            pos += direction.as_xy();
        }
        self.tokens
            .push(PositionedToken::new(token, start, direction));
    }

    fn set_letter(&mut self, letter: Letter, pos: XY) {
        let prev_letter = self.letter_by_pos.insert(pos, letter);
        assert!(prev_letter == None || prev_letter == Some(letter));
        if prev_letter.is_none() {
            self.pos_by_letter.entry(letter).or_default().push(pos);
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (x_limits, y_limits) = self.space();

        for y in y_limits {
            for x in x_limits.clone() {
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
