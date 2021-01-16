use crate::models::time::Time;

pub mod english;
pub mod french;
pub mod portuguese;

/// Represents a possible language, that can spell out any valid time
pub trait Language {
    fn spell(&self, time: Time) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::english::English;
    use crate::languages::french::French;
    use crate::languages::portuguese::Portuguese;

    #[test]
    fn debug_languages() {
        println!("English");
        for time in Time::all_times() {
            println!("{}: {}", time, English.spell(time));
        }
        println!("French");
        for time in Time::all_times() {
            println!("{}: {}", time, French.spell(time));
        }
        println!("Portuguese");
        for time in Time::all_times() {
            println!("{}: {}", time, Portuguese.spell(time));
        }
    }
}
