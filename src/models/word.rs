use anyhow::{ensure, Error};
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt;

/// Represents a non-empty list of letters
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Word {
    letters: Vec<Letter>,
}

/// Represents a letter than can be put in a word grid
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

#[allow(clippy::len_without_is_empty)]
impl Word {
    pub fn len(&self) -> usize {
        self.letters.len()
    }

    pub fn letters(&self) -> &[Letter] {
        &self.letters
    }
}

impl TryFrom<char> for Letter {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Letter::A),
            'B' => Ok(Letter::B),
            'C' => Ok(Letter::C),
            'D' => Ok(Letter::D),
            'E' => Ok(Letter::E),
            'F' => Ok(Letter::F),
            'G' => Ok(Letter::G),
            'H' => Ok(Letter::H),
            'I' => Ok(Letter::I),
            'J' => Ok(Letter::J),
            'K' => Ok(Letter::K),
            'L' => Ok(Letter::L),
            'M' => Ok(Letter::M),
            'N' => Ok(Letter::N),
            'O' => Ok(Letter::O),
            'P' => Ok(Letter::P),
            'Q' => Ok(Letter::Q),
            'R' => Ok(Letter::R),
            'S' => Ok(Letter::S),
            'T' => Ok(Letter::T),
            'U' => Ok(Letter::U),
            'V' => Ok(Letter::V),
            'W' => Ok(Letter::W),
            'X' => Ok(Letter::X),
            'Y' => Ok(Letter::Y),
            'Z' => Ok(Letter::Z),
            _ => Err(Error::msg(format!(
                "Impossible to convert {} as letter",
                value
            ))),
        }
    }
}

impl<'a> TryFrom<&'a str> for Word {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let letters: Vec<_> = value.chars().map(Letter::try_from).try_collect()?;
        ensure!(!letters.is_empty());
        Ok(Word { letters })
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Letter::A => 'A',
                Letter::B => 'B',
                Letter::C => 'C',
                Letter::D => 'D',
                Letter::E => 'E',
                Letter::F => 'F',
                Letter::G => 'G',
                Letter::H => 'H',
                Letter::I => 'I',
                Letter::J => 'J',
                Letter::K => 'K',
                Letter::L => 'L',
                Letter::M => 'M',
                Letter::N => 'N',
                Letter::O => 'O',
                Letter::P => 'P',
                Letter::Q => 'Q',
                Letter::R => 'R',
                Letter::S => 'S',
                Letter::T => 'T',
                Letter::U => 'U',
                Letter::V => 'V',
                Letter::W => 'W',
                Letter::X => 'X',
                Letter::Y => 'Y',
                Letter::Z => 'Z',
            }
        )
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.letters.iter().format(""))
    }
}
