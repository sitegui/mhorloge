use crate::models::time::Time;

pub fn spell(time: Time) -> String {
    let hours = time.hours();
    let minutes = time.minutes();
    let next_hour = (hours + 1) % 24;
    match minutes {
        0 => spell_hours(hours, true),
        1..=14 => format!(
            "{} NACH {}",
            spell_number(minutes),
            spell_hours(hours, false)
        ),
        15 => format!("VIERTEL NACH {}", spell_hours(hours, false)),
        16..=29 => format!(
            "{} VOR HALB {}",
            spell_number(30 - minutes),
            spell_hours(next_hour, false)
        ),
        30 => format!("HALB {}", spell_hours(next_hour, false)),
        31..=44 => format!(
            "{} NACH HALB {}",
            spell_number(45 - minutes),
            spell_hours(next_hour, false)
        ),
        45 => format!("VIERTEL VOR {}", spell_hours(next_hour, false)),
        46..=59 => format!(
            "{} VOR {}",
            spell_number(60 - minutes),
            spell_hours(next_hour, false)
        ),
        _ => unreachable!(),
    }
}

fn spell_hours(n: u8, include_um: bool) -> String {
    assert!(n < 24);

    match (n, include_um) {
        (0, _) => "MITTERNACHT".to_owned(),
        (12, _) => "MITTAG".to_owned(),
        (n, true) if n < 12 => format!("UM {}", spell_number(n)),
        (n, false) if n < 12 => spell_number(n),
        (n, include_o_clock) => spell_hours(n - 12, include_o_clock),
    }
}

fn spell_number(n: u8) -> String {
    assert!(n < 15);

    let solo = &[
        "", "EINS", "ZWEI", "DREI", "VIER", "FUNF", "SECHS", "SIEBEN", "ACHT", "NEUN", "ZEHN",
        "ELF", "ZWOLF", "DREIZEHN", "VIERZEHN",
    ];

    solo[n as usize].to_owned()
}
