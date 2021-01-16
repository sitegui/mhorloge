use crate::languages::Language;
use crate::models::time::Time;

pub struct French;

impl Language for French {
    fn spell(&self, time: Time) -> String {
        match (time.hours(), time.minutes()) {
            (hours, 0) => spell_hours(hours),
            (hours, 15) => format!("{} ET QUART", spell_hours(hours)),
            (hours, 30) => format!("{} ET DEMIE", spell_hours(hours)),
            (hours, 45) => format!("{} MOINS LE QUART", spell_hours((hours + 1) % 24)),
            (hours, minutes) if minutes < 30 => {
                format!("{} {}", spell_hours(hours), spell_number(minutes, false))
            }
            (hours, minutes) => format!(
                "{} MOINS {}",
                spell_hours((hours + 1) % 24),
                spell_number(60 - minutes, false),
            ),
        }
    }
}

fn spell_hours(n: u8) -> String {
    assert!(n < 24);

    match n {
        0 => "MINUIT".to_owned(),
        1 => "UNE HEURE".to_owned(),
        12 => "MIDI".to_owned(),
        n if n < 12 => format!("{} HEURES", spell_number(n, false)),
        n => spell_hours(n - 12),
    }
}

fn spell_number(n: u8, masculine: bool) -> String {
    assert!(n < 60);

    let solo = &[
        "",
        "SPECIAL_CASE",
        "DEUX",
        "TROIS",
        "QUATRE",
        "CINQ",
        "SIX",
        "SEPT",
        "HUIT",
        "NEUF",
        "DIX",
        "ONZE",
        "DOUZE",
        "TREIZE",
        "QUATORZE",
        "QUINZE",
        "SEIZE",
    ];

    let composed = &["DIX", "VINGT", "TRENTE", "QUARANTE", "CINQUANTE"];

    match (n, masculine) {
        (1, true) => "UN".to_owned(),
        (1, false) => "UNE".to_owned(),
        (n, _) if n < 17 => solo[n as usize].to_owned(),
        (n, _) if n % 10 == 0 => composed[(n / 10 - 1) as usize].to_owned(),
        (n, masculine) => format!(
            "{}{}{}",
            composed[(n / 10 - 1) as usize],
            if n % 10 == 1 { " ET " } else { " " },
            spell_number(n % 10, masculine)
        ),
    }
}
