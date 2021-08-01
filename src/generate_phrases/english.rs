use crate::models::time::Time;

pub fn spell(time: Time) -> String {
    match (time.hours(), time.minutes()) {
        (hours, 0) => spell_hours(hours, true),
        (hours, 15) => format!("QUARTER PAST {}", spell_hours(hours, false)),
        (hours, 30) => format!("HALF PAST {}", spell_hours(hours, false)),
        (hours, 45) => format!("QUARTER TO {}", spell_hours((hours + 1) % 24, false)),
        (hours, minutes) if minutes < 30 => format!(
            "{} PAST {}",
            spell_number(minutes),
            spell_hours(hours, false)
        ),
        (hours, minutes) => format!(
            "{} TO {}",
            spell_number(60 - minutes),
            spell_hours((hours + 1) % 24, false),
        ),
    }
}

fn spell_hours(n: u8, include_o_clock: bool) -> String {
    assert!(n < 24);

    match (n, include_o_clock) {
        (0, _) => "MIDNIGHT".to_owned(),
        (12, _) => "MIDDAY".to_owned(),
        (n, true) if n < 12 => format!("{} O CLOCK", spell_number(n)),
        (n, false) if n < 12 => spell_number(n),
        (n, include_o_clock) => spell_hours(n - 12, include_o_clock),
    }
}

fn spell_number(n: u8) -> String {
    assert!(n < 60);

    let solo = &[
        "",
        "ONE",
        "TWO",
        "THREE",
        "FOUR",
        "FIVE",
        "SIX",
        "SEVEN",
        "EIGHT",
        "NINE",
        "TEN",
        "ELEVEN",
        "TWELVE",
        "THIRTEEN",
        "FOURTEEN",
        "FIFTEEN",
        "SIXTEEN",
        "SEVENTEEN",
        "EIGHTEEN",
        "NINETEEN",
    ];

    let composed = &["TWENTY", "THIRTY", "FORTY", "FIFTY"];

    match n {
        n if n < 20 => solo[n as usize].to_owned(),
        n if n % 10 == 0 => composed[(n / 10 - 2) as usize].to_owned(),
        n => format!(
            "{} {}",
            composed[(n / 10 - 2) as usize],
            spell_number(n % 10)
        ),
    }
}
