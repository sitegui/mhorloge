use crate::models::letter::Letter;
use crate::models::position_restriction::PositionRestriction;
use crate::models::positioned_token::{Direction, OrientedToken, PositionedToken, XY};
use crate::models::token::Token;
use crate::models::token_relations::TokenRelations;
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::fmt::Write;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Grid {
    letter_by_pos: HashMap<XY, Letter>,
    tokens: Vec<PositionedToken>,
    /// The extremes of the bounding rectangle of the inserted letters. This rectangle does not
    /// depend on the desired aspect ratio.
    top_left: XY,
    bottom_right: XY,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            letter_by_pos: HashMap::new(),
            tokens: Vec::new(),
            top_left: XY::new(i32::MAX, i32::MAX),
            bottom_right: XY::new(i32::MIN, i32::MIN),
        }
    }

    /// Return the number of determined letters of this grid
    pub fn num_letters(&self) -> i32 {
        self.letter_by_pos.len() as i32
    }

    /// Return the bounding box of this grid
    pub fn space(&self) -> (RangeInclusive<i32>, RangeInclusive<i32>) {
        (
            self.top_left.x..=self.bottom_right.x,
            self.top_left.y..=self.bottom_right.y,
        )
    }

    /// Returns `(width, height)` of the bounding box of the grid
    pub fn size(&self) -> (i32, i32) {
        let (x, y) = self.space();
        let width = x.end() - x.start();
        let height = y.end() - y.start();
        (width, height)
    }

    fn pos_by_letter(&self, letter: Letter) -> impl Iterator<Item = XY> + '_ {
        self.letter_by_pos
            .iter()
            .filter_map(move |(&pos, &some_letter)| {
                if some_letter == letter {
                    Some(pos)
                } else {
                    None
                }
            })
    }

    /// Return all resulting grids for the valid insertions of the given token
    pub fn enumerate_insertions(
        &self,
        relations: &TokenRelations,
        token: &Token,
        allow_diagonal: bool,
    ) -> Vec<Grid> {
        // Enumerate all insertions that use a valid pivot. A `BTreeSet` is used to deduplicate
        // them, in case a single insertion covers multiple pivots simultaneously
        let mut insertions = BTreeSet::new();

        for oriented in OrientedToken::orientations(token, allow_diagonal) {
            let restrictions = PositionRestriction::new(relations, &self.tokens, oriented);

            // Test insertions that use a pivot
            for (letter_index, &letter) in token.text.letters().iter().enumerate() {
                let n = letter_index as i32;

                for pivot in self.pos_by_letter(letter) {
                    let start = pivot - oriented.direction().as_xy() * n;
                    let positioned = PositionedToken::new(oriented, start);
                    if restrictions.is_valid_start(start) && self.check_letters(token, positioned) {
                        insertions.insert(positioned);
                    }
                }
            }

            // Test insertions that do not use any pivot: find a valid first insertion and also try
            // a more spaced one
            for scan_dir in [
                Direction::Horizontal,
                Direction::Vertical,
                Direction::Diagonal,
            ] {
                let mut start = restrictions.min_start().unwrap_or(XY::ORIGIN);
                loop {
                    let positioned = PositionedToken::new(oriented, start);
                    if restrictions.is_valid_start(start) && self.check_letters(token, positioned) {
                        insertions.insert(positioned);

                        let start_2 = start + scan_dir.as_xy() * 3;
                        let positioned_2 = PositionedToken::new(oriented, start_2);
                        if restrictions.is_valid_start(start_2)
                            && self.check_letters(token, positioned_2)
                        {
                            insertions.insert(positioned_2);
                        }
                        break;
                    }

                    start += scan_dir.as_xy();
                }
            }
        }

        // Collect the valid insertions
        insertions
            .into_par_iter()
            .map(|positioned| {
                let mut grid = self.clone();
                grid.insert(token, positioned);
                grid
            })
            .collect()
    }

    pub fn tokens(&self) -> &[PositionedToken] {
        &self.tokens
    }

    pub fn get(&self, at: XY) -> Option<Letter> {
        self.letter_by_pos.get(&at).copied()
    }

    fn insert(&mut self, token: &Token, positioned: PositionedToken) {
        for (pos, letter) in positioned.iter(token) {
            let prev_letter = self.letter_by_pos.insert(pos, letter);
            assert!(prev_letter == None || prev_letter == Some(letter));

            self.top_left.x = self.top_left.x.min(pos.x);
            self.top_left.y = self.top_left.y.min(pos.y);
            self.bottom_right.x = self.bottom_right.x.max(pos.x);
            self.bottom_right.y = self.bottom_right.y.max(pos.y);
        }

        self.tokens.push(positioned);
    }

    /// Check if the token can be "printed" in the given position and respect the existing letters
    fn check_letters(&self, token: &Token, positioned: PositionedToken) -> bool {
        for (xy, new_letter) in positioned.iter(token) {
            let current_letter = self.letter_by_pos.get(&xy).copied();

            if current_letter != None && current_letter != Some(new_letter) {
                return false;
            }
        }

        true
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
