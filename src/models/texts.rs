use itertools::Itertools;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Texts {
    tags: BTreeMap<String, TextTag>,
    texts: Vec<String>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextTag {
    id: TextTagId,
    len: u8,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextTagId(u16);

impl Texts {
    pub fn new() -> Self {
        Texts {
            tags: BTreeMap::new(),
            texts: Vec::new(),
        }
    }

    pub fn encode(&mut self, text: &str) -> TextTag {
        self.tags.get(text).copied().unwrap_or_else(|| {
            let tag = TextTag {
                id: TextTagId(self.tags.len().try_into().unwrap()),
                len: text.len().try_into().unwrap(),
            };
            self.tags.insert(text.to_owned(), tag);
            self.texts.push(text.to_owned());
            tag
        })
    }

    pub fn decode(&self, tag: TextTag) -> &str {
        &self.texts[tag.id.0 as usize]
    }
}

impl fmt::Display for Texts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.texts.iter().format(", "))
    }
}

#[allow(clippy::len_without_is_empty)]
impl TextTag {
    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl Default for Texts {
    fn default() -> Self {
        Self::new()
    }
}
