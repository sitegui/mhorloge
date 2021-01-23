use crate::clusterize::constraints::{Constraints, Order};
use crate::clusterize::position::Position;
use crate::clusterize::rotated_cluster::RotatedCluster;
use crate::clusterize::token_in_cluster::TokenInCluster;
use crate::models::texts::{TextTag, Texts};
use crate::tokenize::TokenId;
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Cluster<'a> {
    used_letters: RefCell<Option<BTreeSet<char>>>,
    letters: BTreeMap<Position, char>,
    tokens: Vec<TokenInCluster>,
    texts: &'a Texts,
    constraints: &'a Constraints,
    can_rotate_once: bool,
    can_rotate_twice: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    Horizontal,
    Diagonal,
    Vertical,
}

impl<'a> Cluster<'a> {
    pub fn new(
        texts: &'a Texts,
        constraints: &'a Constraints,
        token_id: TokenId,
        text: TextTag,
    ) -> Self {
        let mut used_letters = BTreeSet::new();
        let mut letters = BTreeMap::new();
        for (j, letter) in texts.decode(text).chars().enumerate() {
            used_letters.insert(letter);
            letters.insert(Position::new(0, j as i16), letter);
        }

        Cluster {
            used_letters: RefCell::new(Some(used_letters)),
            letters,
            tokens: vec![TokenInCluster {
                id: token_id,
                text,
                direction: Direction::Horizontal,
                start: Position::new(0, 0),
            }],
            texts,
            constraints,
            can_rotate_once: true,
            can_rotate_twice: true,
        }
    }

    pub fn a_common_letter(&self, other: &Cluster, rng: &mut SmallRng) -> Option<char> {
        self.used_letters()
            .intersection(&other.used_letters())
            .choose(rng)
            .copied()
    }

    pub fn a_position(&self, letter: char, rng: &mut SmallRng) -> Position {
        self.letters
            .iter()
            .filter(|&(_, &self_letter)| self_letter == letter)
            .map(|(&pos, _)| pos)
            .choose(rng)
            .unwrap()
    }

    pub fn all_superposed(
        &'a self,
        other: &'a Cluster<'a>,
        pos_self: Position,
        pos_other: Position,
    ) -> Vec<Self> {
        let mut results = vec![];

        let rotated_other = RotatedCluster::new(other);

        // No relative rotation
        self.push_superposed(rotated_other, pos_self, pos_other, &mut results);

        if let Some(rotated_other) = rotated_other.rotated() {
            // `other` rotated once
            self.push_superposed(rotated_other, pos_self, pos_other, &mut results);

            if let Some(rotated_other) = rotated_other.rotated() {
                // `other` rotated twice
                self.push_superposed(rotated_other, pos_self, pos_other, &mut results);
            }
        }

        let rotated_self = RotatedCluster::new(self);
        if let Some(rotated_self) = rotated_self.rotated() {
            // `self` rotated once
            other.push_superposed(rotated_self, pos_other, pos_self, &mut results);

            // `self` rotated twice
            if let Some(rotated_self) = rotated_self.rotated() {
                other.push_superposed(rotated_self, pos_other, pos_self, &mut results);
            }
        }

        results
    }

    pub fn can_rotate_once(&self) -> bool {
        self.can_rotate_once
    }

    pub fn can_rotate_twice(&self) -> bool {
        self.can_rotate_twice
    }

    pub fn tokens(&self) -> &[TokenInCluster] {
        &self.tokens
    }

    pub fn constraints(&self) -> &'a Constraints {
        &self.constraints
    }

    fn push_superposed(
        &self,
        other: RotatedCluster<'a>,
        pos_self: Position,
        original_pos_other: Position,
        results: &mut Vec<Self>,
    ) {
        // Transform other tokens
        let pos_other = other.transform(original_pos_other);
        let delta = pos_other - pos_self;
        let new_tokens = other.tokens().map(|mut token| {
            token.start -= delta;
            token
        });
        let new_tokens_len = new_tokens.len();

        // Check constraints
        // TODO: maybe use `tuple_combinations()` instead of `cartesian_product()`?
        for (new_token, &self_token) in new_tokens.clone().cartesian_product(&self.tokens) {
            let constraint = self.constraints.get(self_token.id, new_token.id);
            if !self_token.respects(new_token, constraint) {
                return;
            }
        }

        // Create result
        let mut result = Cluster {
            used_letters: RefCell::new(None),
            letters: self.letters.clone(),
            tokens: Vec::with_capacity(self.tokens.len() + new_tokens_len),
            texts: self.texts,
            constraints: self.constraints,
            can_rotate_once: self.can_rotate_once && other.can_rotate_once(),
            can_rotate_twice: self.can_rotate_twice && other.can_rotate_twice(),
        };
        result.tokens.extend_from_slice(&self.tokens);

        for new_token in new_tokens {
            for (pos, letter) in new_token.letters(result.texts) {
                let old_letter = result.letters.insert(pos, letter);
                if old_letter.is_some() && old_letter != Some(letter) {
                    // Tried to overwrite a different letter
                    return;
                }
            }
            result.tokens.push(new_token);
        }

        // Check if the rotated version respect the constraints
        if let Some(rotated_once) = RotatedCluster::new(&result).rotated() {
            let can_rotate_once = rotated_once.is_valid();

            if let Some(rotated_twice) = rotated_once.rotated() {
                if !rotated_twice.is_valid() {
                    result.can_rotate_twice = false;
                }
            }

            // Use a variable to workaround borrow checker
            result.can_rotate_once = can_rotate_once;
        }

        results.push(result);
    }

    fn used_letters(&self) -> RefMut<BTreeSet<char>> {
        RefMut::map(self.used_letters.borrow_mut(), |used_letters| {
            used_letters.get_or_insert_with(|| self.letters.values().copied().collect())
        })
    }
}

impl<'a> fmt::Display for Cluster<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Print letters
        let (min_i, max_i) = self
            .letters
            .keys()
            .map(|pos| pos.i)
            .minmax()
            .into_option()
            .unwrap();
        let (min_j, max_j) = self
            .letters
            .keys()
            .map(|pos| pos.j)
            .minmax()
            .into_option()
            .unwrap();
        for i in min_i..=max_i {
            for j in min_j..=max_j {
                let pos = Position::new(i, j);
                let letter = self.letters.get(&pos).copied().unwrap_or('.');
                write!(f, "{}", letter)?;
            }
            writeln!(f)?;
        }

        // Print tokens info
        writeln!(f, "Constraints:")?;
        for (i, token_a) in self.tokens.iter().enumerate() {
            let constraints = self
                .tokens
                .iter()
                .skip(i + 1)
                .filter_map(|token_b| {
                    let constraint = self.constraints.get(token_a.id, token_b.id);
                    if constraint.coexist || constraint.order != Order::None {
                        Some((self.texts.decode(token_b.text), constraint))
                    } else {
                        None
                    }
                })
                .collect_vec();

            if !constraints.is_empty() {
                writeln!(
                    f,
                    "\t{}: {}",
                    self.texts.decode(token_a.text),
                    constraints
                        .into_iter()
                        .format_with(", ", |(token_b, constraint), f| {
                            f(&format_args!("{} {}", constraint, token_b))
                        })
                )?;
            }
        }

        // Print rotation info
        writeln!(f, "Can rotate once = {}", self.can_rotate_once)?;
        writeln!(f, "Can rotate twice = {}", self.can_rotate_twice)?;

        Ok(())
    }
}

impl Direction {
    /// Return the unit position in this direction
    pub fn unit(self) -> Position {
        match self {
            Direction::Horizontal => Position::new(0, 1),
            Direction::Diagonal => Position::new(1, 1),
            Direction::Vertical => Position::new(1, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clusterize::constraints::tests::tokenize_example;

    #[test]
    fn test() {
        let tokenize = tokenize_example();
        let constraints = Constraints::new(&tokenize);

        let mut texts = Texts::new();
        let text_tags = tokenize
            .tokens
            .iter()
            .map(|token| texts.encode(&token.text))
            .collect_vec();

        let elephant = Cluster::new(&texts, &constraints, TokenId(1), text_tags[1]);
        let spider = Cluster::new(&texts, &constraints, TokenId(3), text_tags[3]);

        let superposed = elephant.all_superposed(&spider, Position::new(0, 3), Position::new(0, 1));

        println!("{}", elephant);

        for each in superposed {
            println!("{}", each);
        }
    }
}
