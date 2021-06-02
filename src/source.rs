use reqwest::{Client, Url};
use select::{
    document::Document,
    predicate::{Child, Descendant, Name, Or, Text},
};
use serde::{Deserialize, Serialize};

use crate::library::BookName;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub location: String,
    html:         Option<String>,
    #[serde(skip)]
    doc:          Option<Document>,
}

#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Debug, Clone)]
pub struct Site {
    pub location: String,
    pub pred:     String,
}

impl Source {
    pub async fn new(url: String) -> Self {
        let (doc, html) = Self::download(&url, &Client::new()).await;
        Self {
            location: url,
            html,
            doc,
        }
    }

    pub async fn get(
        &self,
        visual: bool,
    ) -> Option<Vec<String>> {
        match visual {
            true => self.images_batch().await,
            false => self.text().await,
        }
    }

    pub async fn download(
        url: &String,
        client: &Client,
    ) -> (Option<Document>, Option<String>) {
        let html = client
            .get(url)
            .send()
            .await
            .ok()
            .unwrap()
            .text()
            .await
            .ok()
            .unwrap();
        (Some(html.clone().as_str().into()), Some(html))
    }

    pub async fn refresh(
        &mut self,
        url: Option<String>,
    ) {
        (self.doc, self.html) =
            Self::download(&url.unwrap_or(self.location.clone()), &Client::new())
                .await;
    }

    #[allow(dead_code)]
    fn find_index(&self) { self.location.parse::<Url>().unwrap().path(); }

    pub async fn check_visual(&self) -> Option<bool> {
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
            (true, true) => self.text().await.unwrap().len() < 20,
            (true, false) => false,
            (false, true) => true,
            (false, false) => self.text().await.unwrap().len() < 20,
        })
    }

    /// Returns something that looks like a book title
    pub async fn title(&self) -> BookName {
        todo!();
        // else Self::default()
    }

    pub async fn pos(&self) -> u16 { 0 }

    /// Returns a Source leading the the index page of the chapter
    pub async fn index(&self) -> Self {
        todo!();
        // else Self::default()
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

    pub async fn next(
        &self,
        pred: &str,
    ) -> Option<String> {
        self.doc.as_ref().and_then(|a| {
            a.select(Child(Name("a"), Text))
                .filter(|a| a.text().contains(pred))
                .map(|a| a.parent().unwrap().attr("href").unwrap().to_string())
                .next()
        })
    }

    /// Returns the text from the children of the <div> with most <p> tags
    pub async fn text(&self) -> Option<Vec<String>> {
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
    pub async fn images_batch(&self) -> Option<Vec<String>> {
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

    pub fn num(&self) -> u16 { todo!() }
}

impl PartialOrd for Source {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(self.location.cmp(&other.location))
    }
}
impl Ord for Source {
    fn cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        self.location.cmp(&other.location)
    }
}
impl Eq for Source {}
impl PartialEq for Source {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.location == other.location && self.html == other.html
    }
}
impl From<String> for Source {
    fn from(url: String) -> Self {
        Self {
            location: url,
            html:     None,
            doc:      None,
        }
    }
}
impl From<&String> for Source {
    fn from(url: &String) -> Self {
        Self {
            location: url.clone(),
            html:     None,
            doc:      None,
        }
    }
}
impl AsRef<Source> for Source {
    fn as_ref(&self) -> &Source { self }
}
