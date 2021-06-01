use reqwest::{Client, Url};
use select::{
    document::Document,
    predicate::{Child, Descendant, Name, Or, Text},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub location: String,
    html:         Option<String>,
    #[serde(skip)]
    doc:          Option<Document>,
}

pub struct Site {
    _pred: String,
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

    pub fn check_visual(&mut self) -> bool {
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
        t.iter().any(|s| f(s)) || p.iter().any(|s| f(s))
    }

    /// Returns the biggest congregation of links in the html
    pub async fn source(&self) -> Self {
        todo!();
        // else Self::default()
    }

    /// Returns the biggest congregation of links in the html
    pub async fn index(&self) -> Option<Vec<String>> {
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
