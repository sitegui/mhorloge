use crate::tokenize::{TokenId, TokenizeOut};
use itertools::Itertools;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Constraints {
    /// The constraint between `a` and `b` is stored at `constraints[a][b]`.
    constraints: Vec<Vec<Constraint>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Constraint {
    pub coexist: bool,
    pub order: Order,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    AThenB,
    BThenA,
    None,
}

impl Constraints {
    pub fn new(tokenize: &TokenizeOut) -> Self {
        let len = tokenize
            .tokens
            .iter()
            .map(|token| token.id.0 as usize)
            .max()
            .unwrap()
            + 1;

        let mut constraints = Constraints {
            constraints: vec![
                vec![
                    Constraint {
                        coexist: false,
                        order: Order::None
                    };
                    len
                ];
                len
            ],
        };

        // Set coexist constraints
        for phrase in &tokenize.phrases {
            for (&token_a, &token_b) in phrase.tokens.iter().tuple_combinations::<(_, _)>() {
                constraints.get_mut(token_a, token_b).coexist = true;
                constraints.get_mut(token_b, token_a).coexist = true;
            }
        }

        // Set ordering constraints
        for token in &tokenize.tokens {
            for &follower in &token.followed_by {
                constraints.get_mut(token.id, follower).order = Order::AThenB;
                constraints.get_mut(follower, token.id).order = Order::BThenA;
            }
        }

        constraints
    }

    pub fn get(&self, a: TokenId, b: TokenId) -> Constraint {
        self.constraints[a.0 as usize][b.0 as usize]
    }

    fn get_mut(&mut self, a: TokenId, b: TokenId) -> &mut Constraint {
        &mut self.constraints[a.0 as usize][b.0 as usize]
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let order = match self.order {
            Order::AThenB => "before",
            Order::BThenA => "after",
            Order::None => "",
        };
        let coexist = match self.coexist {
            true => "+coexist",
            false => "",
        };
        write!(f, "{}{}", order, coexist)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::generate_phrases::PhraseId;
    use crate::tokenize::{TokenizeOutEl, TokenizeOutPhraseEl};

    pub fn tokenize_example() -> TokenizeOut {
        TokenizeOut {
            tokens: vec![
                TokenizeOutEl {
                    id: TokenId(0),
                    text: "MONKEY".to_string(),
                    followed_by: vec![TokenId(1), TokenId(2), TokenId(3)],
                },
                TokenizeOutEl {
                    id: TokenId(1),
                    text: "ELEPHANT".to_string(),
                    followed_by: vec![TokenId(2), TokenId(3)],
                },
                TokenizeOutEl {
                    id: TokenId(2),
                    text: "TIGER".to_string(),
                    followed_by: vec![TokenId(3)],
                },
                TokenizeOutEl {
                    id: TokenId(3),
                    text: "SPIDER".to_string(),
                    followed_by: vec![],
                },
                TokenizeOutEl {
                    id: TokenId(4),
                    text: "SNAKE".to_string(),
                    followed_by: vec![],
                },
            ],
            phrases: vec![
                TokenizeOutPhraseEl {
                    id: PhraseId(0),
                    tokens: vec![TokenId(0), TokenId(1), TokenId(2)],
                },
                TokenizeOutPhraseEl {
                    id: PhraseId(1),
                    tokens: vec![TokenId(3)],
                },
                TokenizeOutPhraseEl {
                    id: PhraseId(2),
                    tokens: vec![TokenId(4)],
                },
            ],
        }
    }

    #[test]
    fn test() {
        let tokenize = tokenize_example();

        let constraints = Constraints::new(&tokenize);

        let check = |a, b, coexist, order, rev_order| {
            let constraint = constraints.get(TokenId(a), TokenId(b));
            assert_eq!(constraint.coexist, coexist);
            assert_eq!(constraint.order, order);

            let constraint = constraints.get(TokenId(b), TokenId(a));
            assert_eq!(constraint.coexist, coexist);
            assert_eq!(constraint.order, rev_order);
        };

        check(0, 1, true, Order::AThenB, Order::BThenA);
        check(0, 2, true, Order::AThenB, Order::BThenA);
        check(0, 3, false, Order::AThenB, Order::BThenA);
        check(0, 4, false, Order::None, Order::None);

        check(1, 2, true, Order::AThenB, Order::BThenA);
        check(1, 3, false, Order::AThenB, Order::BThenA);
        check(1, 4, false, Order::None, Order::None);

        check(2, 3, false, Order::AThenB, Order::BThenA);
        check(2, 4, false, Order::None, Order::None);

        check(3, 4, false, Order::None, Order::None);
    }
}
