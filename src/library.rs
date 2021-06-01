use crate::source::Source;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
    u8,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub books: BTreeMap<BookName, (Book, Source)>,
}
#[derive(Default, Ord, PartialOrd, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub name: BookName,
    index:    Source,
    chapters: BTreeMap<u16, Chapter>,
    visual:   bool,
    pub pos:  u16,
}
//TODO: implement Default Chapter
#[derive(
    Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub struct Chapter {
    page:    Source,
    content: BTreeMap<u16, Content>,
    pub pos: u16,
}
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
//TODO: implement Default Content
#[derive(
    Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub struct Content(u16, PathBuf);

impl Library {
    pub async fn get(
        &self,
        book: &BookName,
    ) -> Option<&(Book, Source)> {
        self.books.get(book)
    }

    pub async fn add_book(
        &mut self,
        book: BookName,
        site: Option<String>,
    ) -> &mut (Book, Source) {
        if let Some(s) = &site {
            let mut src: Source = s.clone().into();
            src.refresh(None).await;
            let b = Book {
                name: book.clone(),
                index: src.source().await,
                ..Default::default()
            };
            self.books.entry(book).or_insert((b, src))
        } else {
            self.books.entry(book).or_default()
        }
    }

    pub async fn remove_book(
        &mut self,
        book: BookName,
    ) {
        self.books.remove(&book);
    }

    pub async fn set_source(
        &mut self,
        book: BookName,
        url: Option<String>,
    ) {
        match (self.books.entry(book), url) {
            (Entry::Occupied(mut e), Some(url)) => {
                e.get_mut().1 = {
                    let mut source: Source = url.into();
                    source.refresh(None).await;
                    source
                };
            }
            (Entry::Occupied(mut e), None) => *e.get_mut() = Default::default(),
            _ => {}
        }
    }
}
impl Book {
    pub fn set_visual(
        &mut self,
        visual: Option<bool>,
    ) {
        self.visual = visual.unwrap_or(self.index.check_visual());
    }

    pub fn add_chapter(
        &mut self,
        ch: Chapter,
    ) -> Option<Chapter> {
        self.chapters.insert(ch.num(), ch)
    }

    pub fn remove_chapter(
        &mut self,
        ch: Chapter,
    ) -> Option<Chapter> {
        self.chapters.remove(&ch.num())
    }

    pub fn seek(
        &mut self,
        chapter: u16,
    ) -> Option<Chapter> {
        let e = self.chapters.get(&chapter).cloned();
        e.is_some().then(|| self.pos = chapter);
        e
    }

    pub fn prev(&mut self) -> Chapter {
        match self.seek(self.pos.saturating_sub(1)) {
            Some(c) => c,
            None => Chapter::default(),
        }
    }

    pub fn next(&mut self) -> Chapter {
        match self.seek(self.pos.saturating_add(1)) {
            Some(c) => c,
            None => Chapter::default(),
        }
    }
}
impl Chapter {
    pub fn add_content(
        &mut self,
        content: Content,
    ) -> Option<Content> {
        self.content.insert(content.0, content)
    }

    pub fn remove_content(
        &mut self,
        content: Content,
    ) -> Option<Content> {
        self.content.remove(&content.0)
    }

    pub fn num(&self) -> u16 { 0 }

    pub fn seek(
        &mut self,
        page: u16,
    ) -> Option<Content> {
        let e = self.content.get(&page).cloned();
        e.is_some().then(|| self.pos = page);
        e
    }

    pub fn prev(&mut self) -> Content {
        match self.seek(self.pos.saturating_sub(1)) {
            Some(c) => c,
            None => Content::default(),
        }
    }

    pub fn next(&mut self) -> Content {
        match self.seek(self.pos.saturating_add(1)) {
            Some(c) => c,
            None => Content::default(),
        }
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
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}
impl From<BookName> for Book {
    fn from(name: BookName) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
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
