pub mod en;
pub mod fr;
pub mod pt_br;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt_br() {
        println!("{:#?}", pt_br::phrases());
    }

    #[test]
    fn fr() {
        println!("{:#?}", fr::phrases());
    }

    #[test]
    fn en() {
        println!("{:#?}", en::phrases());
    }
}
