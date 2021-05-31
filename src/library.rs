use crate::source::Source;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    convert::Infallible,
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
    str::FromStr,
    u8,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub books: BTreeMap<Book, Source>,
}
#[derive(Default, Ord, PartialOrd, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub name: BookName,
    chapters: BTreeMap<u16, Chapter>,
    visual:   bool,
    position: u16,
}
//TODO: implement Default Chapter
#[derive(
    Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub struct Chapter {
    page:     Source,
    content:  BTreeMap<u8, Content>,
    position: u16,
}
//TODO: implement Default Content
#[derive(
    Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub struct Content(PathBuf);
#[derive(
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Default,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct BookName(String);

impl Library {
    pub fn add_book(
        &mut self,
        book: String,
        site: Option<String>,
    ) {
        let mut book: Book = book.into();
        site.clone().and_then(|a| Some(book.check_visual(&a)));
        if let Entry::Occupied(mut e) = self.books.entry(book) {
            *e.get_mut() = site.and_then(|a| Some(a.into())).unwrap();
        };
    }

    pub fn set_source(
        &mut self,
        book: String,
        url: Option<String>,
    ) {
        match (self.books.entry(book.into()), url) {
            (Entry::Occupied(mut e), Some(u)) => *e.get_mut() = u.into(),
            (Entry::Occupied(mut e), None) => *e.get_mut() = Default::default(),
            _ => {}
        }
    }
}
impl Book {
    pub fn check_visual(
        &mut self,
        url: &String,
    ) {
        let t = vec!["novel", "royalroad", "comrademao"];
        let p = vec!["manga", "hentai", "pururin", "luscious"];
        let f = |s: &&str| -> bool {
            url.parse::<Url>()
                .unwrap()
                .origin()
                .ascii_serialization()
                .contains(s)
        };
        self.visual = t.iter().any(|s| f(s)) || p.iter().any(|s| f(s));
    }

    pub fn set_visual(
        &mut self,
        visual: bool,
    ) {
        self.visual = visual;
    }

    pub fn get(
        &mut self,
        chapter: u16,
    ) -> Option<Chapter> {
        let e = self.chapters.get(&chapter).cloned();
        e.is_some().then(|| self.position = chapter);
        e
    }
}
impl Chapter {
    #[allow(dead_code)]
    fn find_index(&self) { self.page.location.parse::<Url>().unwrap().path(); }

    pub fn get(
        &mut self,
        page: u8,
    ) -> Option<Content> {
        let e = self.content.get(&page).cloned();
        e.is_some().then(|| self.position = page as u16);
        e
    }
}
impl PartialEq for Book {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.name == other.name
    }
}
impl Hash for Book {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.name.hash(state);
    }
}

impl From<String> for Book {
    fn from(name: String) -> Self {
        let mut book: Self = name.clone().into();
        book.check_visual(&name);
        book
    }
}
impl From<String> for BookName {
    fn from(name: String) -> Self { Self(name) }
}
impl Deref for Book {
    type Target = BookName;

    fn deref<'a>(&'a self) -> &'a BookName { &self.name }
}
impl Deref for BookName {
    type Target = String;

    fn deref<'a>(&'a self) -> &'a String { &self.0 }
}
impl FromStr for BookName {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BookName(s.to_string()))
    }
}
