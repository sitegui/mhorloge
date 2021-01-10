use csv::Writer;
use itertools::Itertools;
use std::collections::BTreeMap;

mod languages;

fn main() {
    let pt_br = languages::pt_br::phrases();
    let fr = languages::fr::phrases();
    let en = languages::en::phrases();

    count_tokens(&pt_br);
    count_tokens(&fr);
    count_tokens(&en);

    let mut all = pt_br.clone();
    all.extend(fr.clone());
    all.extend(en.clone());
    count_tokens(&all);

    let mut writer = Writer::from_path("data/phrases.csv").unwrap();
    writer.write_record(&["time", "pt-br", "fr", "en"]).unwrap();

    for hours in 0..24 {
        for minutes in 0..60 {
            let time = format!("{:02}:{:02}", hours, minutes);
            let i = 60 * hours + minutes;
            writer
                .write_record(&[&time, &pt_br[i], &fr[i], &en[i]])
                .unwrap();
        }
    }

    writer.flush().unwrap();
}

fn count_tokens(phrases: &[String]) {
    let mut words: BTreeMap<&str, usize> = BTreeMap::new();
    for phrase in phrases {
        let mut phrase_words: BTreeMap<&str, usize> = BTreeMap::new();
        for word in phrase.split(' ') {
            *phrase_words.entry(word).or_default() += 1
        }

        for (word, count) in phrase_words {
            let current_count = words.entry(word).or_default();
            *current_count = count.max(*current_count);
        }
    }

    println!("Words = {}", words.values().sum::<usize>());
    println!(
        "Letters = {}",
        words
            .iter()
            .map(|(word, count)| word.len() * count)
            .sum::<usize>()
    );

    println!(
        "{}",
        words
            .iter()
            .map(|(word, &count)| if count == 1 {
                format!("{}", word)
            } else {
                format!("{} x {}", count, word)
            })
            .format(", ")
    );
}
