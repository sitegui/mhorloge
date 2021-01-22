use crate::generate_phrases::time::Time;

pub fn spell(time: Time) -> String {
    match (time.hours(), time.minutes()) {
        (hours, 0) => spell_hours(hours),
        (hours, 30) => format!("{} E MEIA", spell_hours(hours)),
        (hours, minutes) if minutes < 30 => {
            format!("{} E {}", spell_hours(hours), spell_number(minutes, true))
        }
        (hours, minutes) => format!(
            "{} PARA {}",
            spell_number(60 - minutes, true),
            spell_hours_with_article((hours + 1) % 24)
        ),
    }
}

fn spell_hours(hours: u8) -> String {
    assert!(hours < 24);

    match hours {
        0 => "MEIA NOITE".to_owned(),
        1 => "UMA HORA".to_owned(),
        12 => "MEIO DIA".to_owned(),
        n if n < 12 => format!("{} HORAS", spell_number(n, false)),
        n => spell_hours(n - 12),
    }
}

fn spell_hours_with_article(hours: u8) -> String {
    assert!(hours < 24);

    match hours {
        0 => "A MEIA NOITE".to_owned(),
        1 => "A UMA HORA".to_owned(),
        12 => "O MEIO DIA".to_owned(),
        n if n < 12 => format!("AS {} HORAS", spell_number(n, false)),
        n => spell_hours_with_article(n - 12),
    }
}

fn spell_number(n: u8, masculine: bool) -> String {
    assert!(n < 60);

    let solo = &[
        "",
        "SPECIAL_CASE",
        "SPECIAL_CASE",
        "TRES",
        "QUATRO",
        "CINCO",
        "SEIS",
        "SETE",
        "OITO",
        "NOVE",
        "DEZ",
        "ONZE",
        "DOZE",
        "TREZE",
        "QUATORZE",
        "QUINZE",
        "DEZESSEIS",
        "DEZESSETE",
        "DEZOITO",
        "DEZENOVE",
    ];

    let composed = &["VINTE", "TRINTA", "QUARENTA", "CINQUENTA"];

    match (n, masculine) {
        (1, true) => "UM".to_owned(),
        (1, false) => "UMA".to_owned(),
        (2, true) => "DOIS".to_owned(),
        (2, false) => "DUAS".to_owned(),
        (n, _) if n < 20 => solo[n as usize].to_owned(),
        (n, _) if n % 10 == 0 => composed[(n / 10 - 2) as usize].to_owned(),
        (n, masculine) => format!(
            "{} E {}",
            composed[(n / 10 - 2) as usize],
            spell_number(n % 10, masculine)
        ),
    }
}
