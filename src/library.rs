use crate::source::Source;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    ops::Deref,
    path::PathBuf,
    u8,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub books: BTreeMap<BookName, Book>,
}
#[derive(Default, Ord, PartialOrd, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub name:  BookName,
    pub index: Source,
    chapters:  BTreeMap<u16, Chapter>,
    visual:    Option<bool>,
    pub pos:   u16,
}
//TODO: implement Default Chapter
#[derive(
    Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub struct Chapter {
    pub page: Source,
    content:  BTreeMap<u16, Content>,
    pub pos:  u16,
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
//TODO: implement Default for Content
#[derive(
    Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub struct Content(pub u16, pub PathBuf);

impl Library {
    pub async fn get(
        &self,
        book: &BookName,
    ) -> Option<&Book> {
        self.books.get(book)
    }

    pub fn rename_book(
        &mut self,
        old: &BookName,
        new_name: BookName,
    ) {
        if self.books.contains_key(old) {
            let mut book = self.books.remove(old).unwrap();
            book.name = new_name.clone();
            self.books.insert(new_name, book);
        }
    }

    pub async fn add_book(
        &mut self,
        book: BookName,
        site: Option<Source>,
    ) -> &mut Book {
        if let Some(src) = site {
            let b = Book {
                name: book.clone(),
                index: src.refresh().await.index().await,
                ..Default::default()
            };
            self.books.entry(book).or_insert(b)
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
                e.get_mut().index = Source::from(url).refresh().await;
            }
            (Entry::Occupied(mut e), None) => *e.get_mut() = Default::default(),
            _ => {}
        }
    }
}
impl Book {
    pub async fn set_visual(
        &mut self,
        visual: Option<bool>,
    ) {
        match visual {
            Some(_) => self.visual = visual,
            None => self.visual = self.index.check_visual().await,
        }
    }

    pub fn visual(&self) -> Option<bool> { self.visual }

    pub async fn add_chapter(
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

    pub fn get(
        &mut self,
        ch: u16,
    ) -> Option<&Chapter> {
        self.chapters.get(&ch)
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

    pub fn get(
        &mut self,
        p: u16,
    ) -> Option<&Content> {
        self.content.get(&p)
    }

    pub fn num(&self) -> u16 { self.page.place.1 }

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
impl Content {
    pub fn save(
        &self,
        data: &[u8],
    ) {
        let pb = &self.1;
        std::fs::create_dir_all(pb).unwrap();
        let pb = &pb.join(format!("{:04}.jpg", self.0));
        File::with_options()
            .write(true)
            .create(true)
            .open(pb)
            .unwrap()
            .write(data)
            .unwrap();
    }

    pub fn file(&self) -> File {
        let pb = &self.1;
        std::fs::create_dir_all(pb).unwrap();
        let pb = &pb.join(format!("{}", self.0));
        std::fs::create_dir_all(pb).unwrap();
        File::with_options()
            .write(true)
            .create(true)
            .open(pb)
            .unwrap()
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
