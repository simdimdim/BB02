use std::time::Duration;

use crate::library::BookName;
use reqwest::{Client, Url};
use select::{
    document::Document,
    predicate::{Child, Descendant, Name, Or, Text},
};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

impl Default for SiteInfo {
    fn default() -> Self {
        Self {
            next: None,
            time: Instant::now(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub location: String,
    html:         Option<String>,
    #[serde(skip)]
    doc:          Option<Document>,
    #[serde(skip)]
    pub place:    (u16, u16, String),
    #[serde(skip)]
    default:      bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SiteInfo {
    pub next: Option<String>,
    pub time: Instant,
}

impl SiteInfo {
    pub fn new() -> Self {
        Self {
            next: None,
            time: Instant::now(),
        }
    }

    pub async fn delay(&mut self) {
        let until = self.time + Duration::from_secs_f32(2.);
        if until > Instant::now() {
            tokio::time::sleep_until(until).await;
        }
        self.time = until;
    }
}

impl Source {
    pub async fn new(url: String) -> Self {
        let mut source = Source::default();
        source.refresh_mut(Some(url)).await;
        source
    }

    pub async fn get(&self, visual: bool) -> Option<Vec<String>> {
        match visual {
            true => self.images_batch(),
            false => self.text(),
        }
    }

    #[inline]
    pub async fn download(
        url: &String, client: Option<&Client>,
    ) -> (Option<Document>, Option<String>) {
        let html = client
            .unwrap_or(&Client::new())
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        (Some(html.clone().as_str().into()), Some(html))
    }

    pub async fn fill(&mut self) {
        (!self.default)
            .then(|| self.refresh_mut(None))
            .unwrap()
            .await;
    }

    fn change_place(&self) -> String {
        let s = if self.check_visual().unwrap() {
            "manga/".to_string()
        } else {
            "novel/".to_string()
        } + self.place.2.as_str();
        s
    }

    pub async fn refresh_mut(&mut self, url: Option<String>) -> &mut Self {
        (self.doc, self.html) =
            Self::download(&url.clone().unwrap_or(self.location.clone()), None)
                .await;
        self.place = get_place(&url.unwrap_or(self.location.clone()));
        self.place.2 = self.change_place();
        self.default = true;
        self
    }

    pub async fn refresh(&self) -> Self {
        let (doc, html) = Self::download(&self.location, None).await;
        let mut place = get_place(&self.location);
        place.2 = self.change_place();
        Self {
            location: self.location.clone(),
            doc,
            html,
            place,
            default: true,
        }
    }

    pub fn check_visual(&self) -> Option<bool> {
        let t = vec!["novel", "royalroad", "comrademao"];
        let p = vec!["manga", "hentai", "pururin", "luscious"];
        let f = |s: &&str| -> bool {
            self.location
                .parse::<Url>()
                .unwrap()
                .origin()
                .ascii_serialization()
                .contains(s)
        };
        Some(match (t.iter().any(|s| f(s)), p.iter().any(|s| f(s))) {
            (true, true) => self.text().unwrap().len() < 20,
            (true, false) => false,
            (false, true) => true,
            (false, false) => self.text().unwrap().len() < 20,
        })
    }

    /// Returns something that looks like a book title
    pub fn title(&self) -> BookName {
        println!("{}", self.doc.is_some());
        let title = self
            .doc
            .as_ref()
            .expect("HTML not found.")
            .select(Name("title"))
            .into_selection()
            .first()
            .unwrap()
            .text();

        if title.contains(" Chapter") {
            title
                .split(" Chapter")
                .filter(|&a| a != "")
                .collect::<Vec<_>>()
                .first()
                .unwrap()
                .to_string()
        } else {
            title
        }
        .into()
        // .to_ascii_lowercase()
        // .split(" chapter")
        // .filter(|&a| a != "")
        // .collect::<Vec<_>>()
        // .first()
        // .unwrap()
        // .chars()
        // .fold(String::new(), |mut acc, s| {
        //     if acc.is_empty() || "- ".contains(acc.chars().last().unwrap()) {
        //         acc.extend(s.to_uppercase());
        //     } else {
        //         acc.push(s);
        //     }
        //     acc
        // })
        // .into()
    }

    pub fn pos(&self) -> u16 { self.place.1 }

    #[allow(dead_code)]
    fn find_index(&self) { self.location.parse::<Url>().unwrap().path(); }

    /// Returns a Source leading the the index page of the chapter
    pub async fn index(&self) -> Self {
        let url = self.location.parse::<Url>().unwrap();
        let base = url.origin().ascii_serialization();
        let mut index = url
            .path_segments()
            .unwrap()
            .rev()
            .fold((Vec::new(), 0, 0), |mut acc, s| {
                if s.to_lowercase().contains("chapter") {
                    acc.1 += 1;
                } else {
                    if acc.1 != 0 || acc.2 > 1 {
                        acc.0.push(s);
                    }
                }
                acc.2 += 1;
                acc
            })
            .0;
        index.push(&base);
        index
            .iter()
            .rev()
            .map(|&a| a)
            .collect::<Vec<_>>()
            .join("/")
            .into()
    }

    /// Returns the biggest congregation of links in the html
    pub async fn chapters(&self) -> Option<Vec<String>> {
        self.doc.as_ref().map(|a| {
            a.select(Descendant(
                Name("div"),
                Or(Name("p"), Or(Name("table"), Name("ul"))),
            ))
            .map(|a| a.select(Name("a")).into_selection())
            .max_by(|a, b| a.len().cmp(&b.len()))
            .unwrap()
            .iter()
            .filter_map(|a| a.attr("href"))
            .map(|a| a.to_string())
            .collect()
        })
        /* TODO: Add a similarity check and only return the biggest cluster of similar
        links */
    }

    pub async fn next(&self, pred: &str) -> Option<Source> {
        let s = self.doc.as_ref().and_then(|a| {
            a.select(Child(Name("a"), Text))
                .filter(|a| a.text().contains(pred))
                .map(|a| {
                    Source::from(
                        a.parent().unwrap().attr("href").unwrap().to_string(),
                    )
                })
                .next()
        });
        match s {
            Some(s) => Some(s.refresh().await),
            None => None,
        }
    }

    /// Returns the text from the children of the <div> with most <p> tags
    pub fn text(&self) -> Option<Vec<String>> {
        self.doc.as_ref().map(|a| {
            // TODO: Improve by par_map()?
            a.select(Child(Name("div"), Name("p")))
                .map(|a| a.parent().unwrap().children().into_selection())
                .max_by(|a, b| a.len().cmp(&b.len()))
                .unwrap()
                .select(Text)
                .iter()
                .map(|a| a.text())
                .collect()
        })
    }

    /// similar to index() return the source addr of the div with most <img>
    pub fn images_batch(&self) -> Option<Vec<String>> {
        self.doc.as_ref().map(|a| {
            a.select(Child(Name("div"), Name("img")))
                .map(|a| a.parent().unwrap().select(Name("img")).into_selection())
                .max_by(|a, b| a.len().cmp(&b.len()))
                .unwrap()
                .iter()
                .map(|a| a.attr("src").unwrap().to_string())
                .collect()
        })
        /* TODO: Similar to index() add a check for links similarity */
    }

    //TODO: Levarage the power of next() to get the whole chapter
    pub async fn images_single(&self) -> Vec<String> {
        match &self.doc {
            Some(_d) => vec![],
            None => vec![],
        }
    }

    pub fn domain(&self) -> String {
        self.location
            .parse::<Url>()
            .unwrap()
            .domain()
            .unwrap()
            .to_string()
    }
}

pub fn get_place(url: &String) -> (u16, u16, String) {
    let url = url.parse::<Url>().expect("Not a Url string.");
    let segments = url
        .path_segments()
        .unwrap()
        .rev()
        .filter(|&a| a != "")
        .collect::<Vec<_>>();
    let numbers = segments
        .iter()
        .map(|a| {
            a.matches(char::is_numeric)
                .collect::<Vec<&str>>()
                .join("")
                .parse::<u16>()
                .unwrap_or_default()
        })
        .collect::<Vec<u16>>();
    // TODO: do a better job at finding the index
    let index_candidate = if segments.len() < 3 {
        segments.iter().last()
    } else {
        segments.iter().rev().skip(1).next()
    };
    match (numbers.as_slice(), index_candidate) {
        ([x @ 0..=9000, y @ 0..=9000, ..], Some(&z)) => (*x, *y, z.to_string()),
        ([x @ 0..=9000], Some(z)) => (0, *x, z.to_string()),
        ([], Some(z)) => (0, 0, z.to_string()),
        _ => (0, 0, "".to_string()),
    }
}

impl Eq for Source {}
impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location && self.html == other.html
    }
}
impl Ord for Source {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.location.cmp(&other.location)
    }
}
impl PartialOrd for Source {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.location.cmp(&other.location))
    }
}
impl From<String> for Source {
    fn from(url: String) -> Self {
        url.parse::<Url>().expect("Couldn't parse.");
        let place = get_place(&url);
        Self {
            location: url.clone(),
            html: None,
            doc: None,
            place,
            default: false,
        }
    }
}
impl From<&String> for Source {
    fn from(url: &String) -> Self {
        url.parse::<Url>().expect("Couldn't parse.");
        let place = get_place(&url);
        Self {
            location: url.clone(),
            html: None,
            doc: None,
            place,
            default: false,
        }
    }
}
impl AsRef<Source> for Source {
    fn as_ref(&self) -> &Source { self }
}
