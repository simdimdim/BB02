use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;
use std::collections::btree_map::Entry;
use std::{collections::BTreeMap, path::PathBuf};

type SiteList = Vec<usize>;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    books: BTreeMap<String, (Book, SiteList)>,
    sites: BTreeMap<String, usize>,
    site_type: BTreeMap<usize, u8>,
}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    visual: bool, //false - text, true = manga
    chapters: Vec<Content>,
}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    n: u16,
    contents: Vec<Content>,
}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Content(u8, PathBuf);

impl Library {
    pub fn add_book(&mut self, book: String, sites: Option<Vec<String>>) {
        let mut v = vec![];
        if let Some(s) = sites {
            for e in s {
                v.push(self.add_site(&e.parse::<Url>().unwrap()));
            }
        }
        if let Entry::Occupied(mut e) = self.books.entry(book) {
            *e.get_mut() = (Book::default(), v);
        };
    }
    pub fn add_source(&mut self, book: String, url: Url) {
        let s = self.add_site(&url);
        if let Entry::Occupied(mut e) = self.books.entry(book.clone()) {
            e.get_mut().1.push(s);
        } else {
            self.add_book(book, Some(vec![url.to_string()]));
        }
        self.sites.insert(url.to_string(), self.sites.len());
    }
    pub fn add_site(&mut self, url: &Url) -> usize {
        let l = self.sites.len();
        *self.sites.entry(url.to_string()).or_insert(l)
    }
    pub fn remove_book(&mut self, book: &String) {
        self.books.remove(book);
    }
    pub fn remove_site(&mut self, book: String, url: &Url) {
        if let Entry::Occupied(mut e) = self.books.entry(book) {
            let sitenum = self.sites.get(&url.to_string()).unwrap();
            e.get_mut().1.retain(|x| x != sitenum);
        }
    }
    pub fn get_type(&mut self, url: &Url) -> u8 {
        let k = self.add_site(&self.parse(url));
        *self.site_type.entry(k).or_default()
    }
    fn parse(&self, url: &Url) -> Url {
        let u1 = url.to_string();
        let u2 = u1.split('/').into_iter().collect::<Vec<&str>>();
        u2[..3].join("/").parse().unwrap()
    }
}
