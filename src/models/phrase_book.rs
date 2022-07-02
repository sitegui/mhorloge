use crate::models::language::Language;
use crate::models::phrase::{Phrase, PhraseId};
use crate::models::text::Text;
use crate::models::time::Time;
use crate::models::word::{Word, WordId};
use std::ops::Index;

/// Represents all phrases that we want to write in the final grid
#[derive(Debug, Clone, Default)]
pub struct PhraseBook {
    phrases: Vec<Phrase>,
    words: Vec<Word>,
}

impl PhraseBook {
    pub fn insert_phrase(&mut self, language: Language, time: Time, phrase: &str) -> PhraseId {
        let mut words = vec![];
        for word in phrase.split(' ') {
            words.push(self.insert_word(Text::new(word)));
        }

        let id = PhraseId(self.phrases.len() as u16);
        self.phrases.push(Phrase {
            id,
            language,
            time,
            words,
        });
        id
    }

    pub fn phrases(&self) -> &[Phrase] {
        &self.phrases
    }

    fn insert_word(&mut self, text: Text) -> WordId {
        let id = WordId(self.words.len() as u16);
        self.words.push(Word { id, text });
        id
    }
}

impl Index<WordId> for PhraseBook {
    type Output = Word;

    fn index(&self, index: WordId) -> &Self::Output {
        &self.words[index.0 as usize]
    }
}
