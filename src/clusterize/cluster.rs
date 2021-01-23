use crate::clusterize::cluster_graph::Constraints;
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
                token: token_id,
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
        if let Some(result) =
            self.superposed(rotated_other, pos_self, rotated_other.transform(pos_other))
        {
            results.push(result);
        }

        if let Some(rotated_other) = rotated_other.rotated() {
            // `other` rotated once
            if let Some(result) =
                self.superposed(rotated_other, pos_self, rotated_other.transform(pos_other))
            {
                results.push(result);
            }

            if let Some(rotated_other) = rotated_other.rotated() {
                // `other` rotated twice
                if let Some(result) =
                    self.superposed(rotated_other, pos_self, rotated_other.transform(pos_other))
                {
                    results.push(result);
                }
            }
        }

        let rotated_self = RotatedCluster::new(self);
        if let Some(rotated_self) = rotated_self.rotated() {
            // `self` rotated once
            if let Some(result) =
                other.superposed(rotated_self, pos_other, rotated_self.transform(pos_self))
            {
                results.push(result);
            }

            // `self` rotated twice
            if let Some(rotated_self) = rotated_self.rotated() {
                if let Some(result) =
                    other.superposed(rotated_self, pos_other, rotated_self.transform(pos_self))
                {
                    results.push(result);
                }
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

    pub fn superposed(
        &self,
        other: RotatedCluster<'a>,
        pos_self: Position,
        pos_other: Position,
    ) -> Option<Self> {
        // Transform other tokens
        let delta = pos_other - pos_self;
        let new_tokens = other.tokens().map(|mut token| {
            token.start -= delta;
            token
        });
        let new_tokens_len = new_tokens.len();

        // Check constraints
        for (new_token, &self_token) in new_tokens.clone().cartesian_product(&self.tokens) {
            let constraint = self.constraints.get(self_token.token, new_token.token);
            if !self_token.respects(new_token, constraint) {
                return None;
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
                    return None;
                }
            }
            result.tokens.push(new_token);
        }

        // TODO: constraint `can_rotate`

        Some(result)
    }

    fn used_letters(&self) -> RefMut<BTreeSet<char>> {
        RefMut::map(self.used_letters.borrow_mut(), |used_letters| {
            used_letters.get_or_insert_with(|| self.letters.values().copied().collect())
        })
    }
}

impl<'a> fmt::Display for Cluster<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
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
