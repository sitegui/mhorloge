use anyhow::{Context, Error};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AspectRatio {
    horizontal: i16,
    vertical: i16,
}

impl AspectRatio {
    /// Return the sides of a rectangle that covers the given rectangle while closely respecting
    /// this ratio.
    pub fn cover(self, width: i16, height: i16) -> (i16, i16) {
        fn ceil_div(a: i16, b: i16) -> i16 {
            (a as f64 / b as f64).ceil() as i16
        }

        // Prefer covering horizontally
        let width_for_ratio = ceil_div(self.horizontal * height, self.vertical);
        if width_for_ratio >= width {
            (width_for_ratio, height)
        } else {
            let height_for_ratio = ceil_div(self.vertical * width, self.horizontal);
            (width, height_for_ratio)
        }
    }
}

impl FromStr for AspectRatio {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (horizontal, vertical) = s.split_once(':').context("Missing colon (:)")?;

        let horizontal = horizontal.parse()?;
        let vertical = vertical.parse()?;

        Ok(AspectRatio {
            horizontal,
            vertical,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            AspectRatio::from_str("4:3").unwrap(),
            AspectRatio {
                horizontal: 4,
                vertical: 3
            }
        );
    }

    #[test]
    fn cover() {
        let ratio = AspectRatio {
            horizontal: 16,
            vertical: 9,
        };

        assert_eq!(ratio.cover(16, 9), (16, 9));

        assert_eq!(ratio.cover(17, 9), (17, 10));
        assert_eq!(ratio.cover(16, 10), (18, 10));

        assert_eq!(ratio.cover(17, 10), (18, 10));
    }
}
