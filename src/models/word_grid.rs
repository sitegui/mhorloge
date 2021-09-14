use crate::models::word::{Letter, Word};
use crate::tokenize::token_graph::TokenSpecId;
use itertools::min;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::{iter, mem};

/// Initial grid size is `2 * INITIAL + 1` rows and columns, centered at zero
const GROW_PADDING: i32 = 8;

/// Represent a square grid composed of letters.
#[derive(Debug, Clone)]
pub struct WordGrid {
    /// The grid in contiguous format: row `r` and column `c` is stored as
    /// `grid[(r + row_offset) * columns + (c + column_offset)]`
    grid: Vec<Option<Letter>>,
    row_offset: i32,
    column_offset: i32,
    rows: i32,
    columns: i32,
    grow_padding: i32,
    tokens: BTreeMap<TokenSpecId, (Position, Orientation)>,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
    Diagonal,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Position {
    pub row: i32,
    pub column: i32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WriteStats {
    pub reused_letters: i32,
    pub new_letters: i32,
    pub empty_neighbors: i32,
}

impl WordGrid {
    /// Create an empty grid, with some pre-reserved space
    pub fn new() -> Self {
        WordGrid::with_grow_padding(GROW_PADDING)
    }

    /// Create an empty grid, with some pre-reserved space
    fn with_grow_padding(grow_padding: i32) -> Self {
        let side = 2 * grow_padding + 1;
        WordGrid {
            grid: vec![None; (side * side) as usize],
            row_offset: grow_padding,
            column_offset: grow_padding,
            rows: side,
            columns: side,
            grow_padding,
            tokens: BTreeMap::new(),
        }
    }

    /// Return the letter at the position, if any. Out of bounds will return `None`.
    pub fn get(&self, position: Position) -> Option<Letter> {
        let r2 = position.row + self.row_offset;
        let c2 = position.column + self.column_offset;
        if r2 < 0 || r2 >= self.rows || c2 < 0 || c2 >= self.columns {
            None
        } else {
            self.grid[(r2 * self.columns + c2) as usize]
        }
    }

    /// Set the letter at the position. When out of bounds, will expand the underlying storage.
    pub fn set(&mut self, position: Position, letter: Letter) {
        let r2 = position.row + self.row_offset;
        let c2 = position.column + self.column_offset;

        // Detect vertical out of bounds and double that side's capacity
        if r2 < 0 {
            self.extend_top(-r2 + self.grow_padding);
        } else if r2 >= self.rows {
            self.extend_bottom(r2 - self.rows + 1 + self.grow_padding);
        }

        // Detect horizontal out of bounds and double that side's capacity
        if c2 < 0 {
            self.extend_left(-c2 + self.grow_padding);
        } else if c2 >= self.columns {
            self.extend_right(c2 - self.columns + 1 + self.grow_padding);
        }

        let r2 = position.row + self.row_offset;
        let c2 = position.column + self.column_offset;
        self.grid[(r2 * self.columns + c2) as usize] = Some(letter);
    }

    pub fn letters(&self) -> impl Iterator<Item = (Position, Letter)> + '_ {
        self.grid.iter().enumerate().filter_map(move |(i, letter)| {
            letter.map(|letter| {
                let c2 = (i as i32) % self.columns;
                let r2 = (i as i32) / self.columns;
                (
                    Position {
                        row: r2 - self.row_offset,
                        column: c2 - self.column_offset,
                    },
                    letter,
                )
            })
        })
    }

    pub fn write_dry_run(
        &self,
        base: Position,
        orientation: Orientation,
        word: &Word,
    ) -> Option<WriteStats> {
        let mut empty_neighbors = HashSet::new();
        let mut positions = HashSet::new();
        let mut stats = WriteStats {
            reused_letters: 0,
            new_letters: 0,
            empty_neighbors: 0,
        };

        for (i, &letter) in word.letters().iter().enumerate() {
            let position = base.advance(orientation, i as i32);
            positions.insert(position);

            match self.get(position) {
                None => {
                    stats.new_letters += 1;
                }
                Some(grid_letter) if grid_letter == letter => {
                    stats.reused_letters += 1;
                }
                // Conflict
                Some(_) => return None,
            }

            for neighbor in position.neighbors() {
                if self.get(neighbor).is_none() {
                    empty_neighbors.insert(neighbor);
                }
            }
        }

        stats.empty_neighbors = empty_neighbors.difference(&positions).count() as i32;
        Some(stats)
    }

    pub fn write(
        &mut self,
        base: Position,
        orientation: Orientation,
        token: TokenSpecId,
        word: &Word,
    ) {
        for (i, &letter) in word.letters().iter().enumerate() {
            let position = base.advance(orientation, i as i32);
            self.set(position, letter);
        }
        self.tokens.insert(token, (base, orientation));
    }

    fn extend_top(&mut self, new_rows: i32) {
        self.row_offset += new_rows;
        self.rows += new_rows;
        let new_cells = (self.columns * new_rows) as usize;
        self.grid.splice(0..0, iter::repeat(None).take(new_cells));
    }

    fn extend_bottom(&mut self, new_rows: i32) {
        self.rows += new_rows;
        self.grid.resize((self.columns * self.rows) as usize, None);
    }

    fn extend_left(&mut self, new_columns: i32) {
        self.column_offset += new_columns;
        self.columns += new_columns;
        let old_grid = mem::replace(
            &mut self.grid,
            Vec::with_capacity((self.rows * self.columns) as usize),
        );
        for row in old_grid.chunks((self.columns - new_columns) as usize) {
            let new_len = self.grid.len() + new_columns as usize;
            self.grid.resize(new_len, None);
            self.grid.extend_from_slice(row);
        }
    }

    fn extend_right(&mut self, new_columns: i32) {
        self.columns += new_columns;
        let old_grid = mem::replace(
            &mut self.grid,
            Vec::with_capacity((self.rows * self.columns) as usize),
        );
        for row in old_grid.chunks((self.columns - new_columns) as usize) {
            self.grid.extend_from_slice(row);
            let new_len = self.grid.len() + new_columns as usize;
            self.grid.resize(new_len, None);
        }
    }
}

impl Position {
    pub fn advance(self, orientation: Orientation, num: i32) -> Self {
        match orientation {
            Orientation::Horizontal => Position {
                row: self.row,
                column: self.column + num,
            },
            Orientation::Vertical => Position {
                row: self.row + num,
                column: self.column,
            },
            Orientation::Diagonal => Position {
                row: self.row + num,
                column: self.column + num,
            },
        }
    }

    pub fn neighbors(self) -> [Self; 8] {
        macro_rules! delta {
            ($row:literal, $column:literal) => {
                Position {
                    row: self.row + $row,
                    column: self.column + $column,
                }
            };
        }
        [
            delta!(-1, -1),
            delta!(-1, 0),
            delta!(-1, 1),
            delta!(0, 1),
            delta!(1, 1),
            delta!(1, 0),
            delta!(1, -1),
            delta!(0, -1),
        ]
    }
}

impl fmt::Display for WordGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Detect bounding box
        let mut min_r2 = self.rows as usize;
        let mut min_c2 = self.columns as usize;
        let mut max_r2 = 0;
        let mut max_c2 = 0;
        for (r2, row) in self.grid.chunks(self.columns as usize).enumerate() {
            for (c2, cell) in row.iter().enumerate() {
                if cell.is_some() {
                    min_r2 = min_r2.min(r2);
                    min_c2 = min_c2.min(c2);
                    max_r2 = max_r2.max(r2);
                    max_c2 = max_c2.max(c2);
                }
            }
        }

        if min_r2 > max_r2 {
            // Empty grid
            return Ok(());
        }

        for row in self
            .grid
            .chunks(self.columns as usize)
            .skip(min_r2)
            .take(max_r2 + 1 - min_r2)
        {
            for cell in row.iter().skip(min_c2).take(max_c2 + 1 - min_c2) {
                match cell {
                    None => write!(f, "."),
                    Some(letter) => write!(f, "{}", letter),
                }?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.row, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::convert::TryFrom;

    #[test]
    fn test() {
        let mut grid = WordGrid::with_grow_padding(2);
        assert_eq!(grid.to_string(), "");
        assert_eq!(grid.letters().collect_vec(), vec![]);

        for orientation in [
            Orientation::Diagonal,
            Orientation::Horizontal,
            Orientation::Vertical,
        ] {
            grid.write(
                Position { row: 0, column: 0 },
                orientation,
                TokenSpecId::new(0),
                &Word::try_from("WORD").unwrap(),
            );
        }
        assert_eq!(grid.to_string(), "WORD\nOO..\nR.R.\nD..D\n");
        assert_eq!(
            grid.letters().collect_vec(),
            [
                (Position { row: 0, column: 0 }, Letter::W),
                (Position { row: 0, column: 1 }, Letter::O),
                (Position { row: 0, column: 2 }, Letter::R),
                (Position { row: 0, column: 3 }, Letter::D),
                (Position { row: 1, column: 0 }, Letter::O),
                (Position { row: 1, column: 1 }, Letter::O),
                (Position { row: 2, column: 0 }, Letter::R),
                (Position { row: 2, column: 2 }, Letter::R),
                (Position { row: 3, column: 0 }, Letter::D),
                (Position { row: 3, column: 3 }, Letter::D)
            ]
        );

        grid.write(
            Position {
                row: -4,
                column: -4,
            },
            Orientation::Diagonal,
            TokenSpecId::new(0),
            &Word::try_from("WORD").unwrap(),
        );
        assert_eq!(
            grid.to_string(),
            "W.......\n.O......\n..R.....\n...D....\n....WORD\n....OO..\n....R.R.\n....D..D\n"
        );

        assert_eq!(
            grid.write_dry_run(
                Position { row: 0, column: 0 },
                Orientation::Horizontal,
                &Word::try_from("WORD").unwrap(),
            ),
            Some(WriteStats {
                reused_letters: 4,
                new_letters: 0,
                empty_neighbors: 11
            })
        );

        assert_eq!(
            grid.write_dry_run(
                Position { row: 0, column: 0 },
                Orientation::Horizontal,
                &Word::try_from("WORM").unwrap(),
            ),
            None
        );
    }
}
