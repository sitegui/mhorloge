use crate::models::word::Word;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Words {
    tag_by_word: BTreeMap<Word, WordTag>,
    words: Vec<Word>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct WordTag {
    id: WordTagId,
    len: u8,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct WordTagId(u16);

impl Words {
    pub fn new() -> Self {
        Words {
            tag_by_word: BTreeMap::new(),
            words: Vec::new(),
        }
    }

    pub fn encode(&mut self, word: Word) -> WordTag {
        self.tag_by_word.get(&word).copied().unwrap_or_else(|| {
            let tag = WordTag {
                id: WordTagId(self.tag_by_word.len().try_into().unwrap()),
                len: word.len().try_into().unwrap(),
            };
            self.tag_by_word.insert(word.clone(), tag);
            self.words.push(word);
            tag
        })
    }

    pub fn decode(&self, tag: WordTag) -> &Word {
        &self.words[tag.id.0 as usize]
    }

    pub fn num_distinct(&self) -> i32 {
        self.words.len() as i32
    }
}

impl fmt::Display for Words {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.words.iter().format(", "))
    }
}

#[allow(clippy::len_without_is_empty)]
impl WordTag {
    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl Default for Words {
    fn default() -> Self {
        Self::new()
    }
}
