use crate::models::positioned_token::{Direction, OrientedToken, PositionedToken};
use crate::models::token_relations::{TokenRelation, TokenRelations};
use crate::XY;
use std::collections::BTreeSet;

/// Represent a space where a token can start, so that the "before" and "after" restrictions are
/// respected
#[derive(Debug, Clone)]
pub struct PositionRestriction {
    min_start: Option<XY>,
    forbidden_starts: BTreeSet<XY>,
}

impl PositionRestriction {
    /// # Panics
    /// It will panic if any of the positioned token is required to be "after" the given token
    pub fn new(
        relations: &TokenRelations,
        positioned_tokens: &[PositionedToken],
        token: OrientedToken,
    ) -> Self {
        let mut forbidden_starts = BTreeSet::new();

        let min_start = positioned_tokens
            .iter()
            .filter_map(
                |positioned| match relations.get(positioned.token_id(), token.token_id()) {
                    TokenRelation::IsAfter => unreachable!(),
                    TokenRelation::None => None,
                    TokenRelation::IsBefore => {
                        let end = positioned.end();

                        // Forbid the grid space that represents the "continuation" of this token.
                        // The token should start at least on the "next" grid space.
                        match positioned.direction() {
                            Direction::Point => {
                                forbidden_starts.insert(end + XY::new(0, 1));
                                forbidden_starts.insert(end + XY::new(1, 1));
                                Some(end + XY::new(2, 0))
                            }
                            Direction::Horizontal => Some(end + XY::new(2, 0)),
                            Direction::Vertical => {
                                forbidden_starts.insert(end + XY::new(0, 1));
                                Some(end + XY::new(1, 0))
                            }
                            Direction::Diagonal => {
                                forbidden_starts.insert(end + XY::new(1, 1));
                                Some(end + XY::new(1, 0))
                            }
                        }
                    }
                },
            )
            .min();

        PositionRestriction {
            min_start,
            forbidden_starts,
        }
    }

    pub fn min_start(&self) -> Option<XY> {
        self.min_start
    }

    pub fn is_valid_start(&self, start: XY) -> bool {
        if let Some(min_start) = self.min_start {
            if start < min_start {
                return false;
            }
        }

        !self.forbidden_starts.contains(&start)
    }
}
