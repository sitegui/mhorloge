use crate::clusterize::cluster::Direction;
use crate::clusterize::cluster_graph::{Constraint, Order};
use crate::clusterize::position::Position;
use crate::models::texts::{TextTag, Texts};
use crate::tokenize::TokenId;

#[derive(Debug, Copy, Clone)]
pub struct TokenInCluster {
    pub token: TokenId,
    pub text: TextTag,
    pub direction: Direction,
    pub start: Position,
}

impl TokenInCluster {
    /// Check if this token and another respect the given constraint.
    pub fn respects(self, other: TokenInCluster, constraint: Constraint) -> bool {
        if constraint.coexist && !self.can_coexist(other) {
            false
        } else {
            match constraint.order {
                Order::AThenB => self.is_before(other),
                Order::BThenA => other.is_before(self),
                Order::None => true,
            }
        }
    }

    pub fn letters<'a>(self, texts: &'a Texts) -> impl Iterator<Item = (Position, char)> + 'a {
        let unit = self.direction.unit();
        texts
            .decode(self.text)
            .chars()
            .enumerate()
            .map(move |(index, char)| (self.start + unit * index as i16, char))
    }

    /// Return whether this token and the other one either:
    /// - do share any letter positions
    /// - share at most one letter position and have different directions
    fn can_coexist(self, other: TokenInCluster) -> bool {
        if self.direction != other.direction {
            // Tokens with different directions share at most one letter. In any case, they can
            // coexist.
            true
        } else {
            // Modify how self and other are viewed so that comparing them is easier.
            // `self` will be seen as being between `(0, 0)` and `(0, len_self - 1)` (inclusive).
            // `other` will be seen as being between `(a, b)` and `(a, b + len_other - 1)` (inclusive).
            let self_start = Position::new(0, 0);
            let other_start = other.start - self.start;
            let self_end = Position::new(0, self.text.len() as i16 - 1);
            let other_end = other_start + Position::new(0, other.text.len() as i16 - 1);

            // Not on the same line: no letter is shared
            other_start.i != self_start.i ||
                // `other` is before `self`
                other_end.j < self_start.j ||
                // `other` is after `self`
                other_start.j > self_end.j
        }
    }

    /// Return if this tokens can be easily identified by a human as being before `other`.
    fn is_before(self, other: TokenInCluster) -> bool {
        let self_end = self.letter_position(self.text.len() - 1);
        // Round up
        let other_middle = other.letter_position((other.text.len() + 1) / 2);
        self_end < other_middle
    }

    fn letter_position(self, index: usize) -> Position {
        self.start + self.direction.unit() * index as i16
    }
}
