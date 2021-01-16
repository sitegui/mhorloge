use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Texts {
    tags: BTreeMap<String, TextTag>,
    texts: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextTag(u16);

impl Texts {
    pub fn new() -> Self {
        Texts {
            tags: BTreeMap::new(),
            texts: Vec::new(),
        }
    }

    pub fn encode(&mut self, text: &str) -> TextTag {
        self.tags.get(text).copied().unwrap_or_else(|| {
            let tag = TextTag(self.tags.len() as u16);
            self.tags.insert(text.to_owned(), tag);
            self.texts.push(text.to_owned());
            tag
        })
    }

    pub fn decode(&self, tag: TextTag) -> &str {
        &self.texts[tag.0 as usize]
    }
}

impl fmt::Display for Texts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.texts.iter().format(", "))
    }
}
