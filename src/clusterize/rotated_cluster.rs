use crate::clusterize::cluster::{Cluster, Direction};
use crate::clusterize::position::Position;
use crate::clusterize::token_in_cluster::TokenInCluster;
use itertools::Itertools;

#[derive(Debug, Copy, Clone)]
pub struct RotatedCluster<'a> {
    cluster: &'a Cluster<'a>,
    rotation: Rotation,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Rotation {
    None,
    Once,
    Twice,
}

impl<'a> RotatedCluster<'a> {
    pub fn new(cluster: &'a Cluster<'a>) -> Self {
        RotatedCluster {
            cluster,
            rotation: Rotation::None,
        }
    }

    pub fn rotated(self) -> Option<Self> {
        match self.rotation {
            Rotation::None if self.cluster.can_rotate_once() => Some(RotatedCluster {
                cluster: self.cluster,
                rotation: Rotation::Once,
            }),
            Rotation::Once if self.cluster.can_rotate_twice() => Some(RotatedCluster {
                cluster: self.cluster,
                rotation: Rotation::Twice,
            }),
            _ => None,
        }
    }

    pub fn tokens(self) -> impl Iterator<Item = TokenInCluster> + Clone + ExactSizeIterator + 'a {
        self.cluster.tokens().iter().map(move |&el| TokenInCluster {
            id: el.id,
            text: el.text,
            direction: self.rotation.new_direction(el.direction),
            start: self.rotation.new_position(el.start),
        })
    }

    pub fn can_rotate_once(self) -> bool {
        match self.rotation {
            Rotation::None => self.cluster.can_rotate_once(),
            Rotation::Once => self.cluster.can_rotate_twice(),
            Rotation::Twice => false,
        }
    }

    pub fn can_rotate_twice(self) -> bool {
        match self.rotation {
            Rotation::None => self.cluster.can_rotate_twice(),
            Rotation::Once | Rotation::Twice => false,
        }
    }

    pub fn transform(self, position: Position) -> Position {
        self.rotation.new_position(position)
    }

    /// Return whether this rotation respects all the constraints. While this is usually the case,
    /// during the construction of a new cluster this "impossible" state can be observed.
    pub fn is_valid(self) -> bool {
        for (token_a, token_b) in self.tokens().tuple_combinations::<(_, _)>() {
            let constraint = self.cluster.constraints().get(token_a.id, token_b.id);
            if !token_a.respects(token_b, constraint) {
                return false;
            }
        }
        true
    }
}

impl Rotation {
    fn new_direction(self, direction: Direction) -> Direction {
        match (self, direction) {
            (Rotation::None, _) => direction,
            (Rotation::Once, Direction::Horizontal) => Direction::Diagonal,
            (Rotation::Once, Direction::Diagonal) => Direction::Vertical,
            (Rotation::Twice, Direction::Horizontal) => Direction::Vertical,
            _ => unreachable!(),
        }
    }

    fn new_position(self, position: Position) -> Position {
        match self {
            Rotation::None => position,
            Rotation::Once => Position::new(position.j, position.j - position.i),
            Rotation::Twice => Position::new(position.j, -position.i),
        }
    }
}
