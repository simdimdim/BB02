use crate::library::Library;
use reqwest::{
    header::{HeaderMap, HeaderName, REFERER},
    Client,
    Url,
};
use select::{
    document::Document,
    predicate::{Child, Name, Text},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs::File,
    io::{BufReader, Write},
};

impl Default for Downloader {
    fn default() -> Self {
        let mut h = BTreeMap::new();
        let mut s = BTreeMap::new();
        s.insert("manganelo".to_string(), vec![0]);
        h.insert(0, Headers::default());
        Self {
            client:     Client::new(),
            headers:    h,
            sites:      s,
            downloader: "/tmp/downloader_headers.json".to_string(),
            lib:        Library::default(),
        }
    }
}
impl Default for Headers {
    fn default() -> Self {
        let mut hm = HeaderMap::new();
        hm.insert(REFERER, "https://manganato.com/".parse().unwrap());
        Self { headers: hm }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Downloader {
    headers:    BTreeMap<u32, Headers>,
    sites:      BTreeMap<String, Vec<u32>>,
    // sites: BTreeMap<String, u32>,
    #[serde(skip)]
    client:     Client,
    #[serde(skip)]
    downloader: String,
    lib:        Library,
}
#[derive(Debug, Serialize, Deserialize)]
struct Headers {
    #[serde(with = "http_serde::header_map")]
    headers: HeaderMap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebPage<'a> {
    location: &'a str,
    #[serde(skip)]
    doc:      Option<Document>,
    html:     String,
}

impl WebPage<'_> {
    pub fn new(html: String) -> Self {
        Self {
            location: "",
            doc: Some(Document::from(html.as_str())),
            html,
        }
    }

    pub async fn text(&self) -> Vec<String> {
        match &self.doc {
            Some(d) => d
                .select(Child(Name("div"), Name("p")))
                // Improved by map_par()?
                .map(|a| a.parent().unwrap().children().into_selection())
                .max_by(|a, b| a.len().cmp(&b.len()))
                .unwrap()
                .select(Text)
                .iter()
                .map(|a| a.text())
                .collect(),
            None => vec![],
        }
    }

    pub async fn images_batch(&self) -> Vec<String> {
        match &self.doc {
            Some(d) => d
                .select(Child(Name("div"), Name("img")))
                .map(|a| a.parent().unwrap().select(Name("img")).into_selection())
                .max_by(|a, b| a.len().cmp(&b.len()))
                .unwrap()
                .iter()
                .map(|a| a.attr("src").unwrap().to_string())
                .collect(),
            None => vec![],
        }
    }

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
        match &self.doc {
            Some(d) => d
                .select(Child(Name("a"), Text))
                .filter(|a| a.text().contains(pred))
                .map(|a| a.parent().unwrap().attr("href").unwrap().to_string())
                .next(),
            None => None,
        }
    }
}

impl Downloader {
    pub async fn download(
        &mut self,
        url: Url,
    ) {
        let html = self
            .client
            .get(url.to_string())
            .send()
            .await
            .ok()
            .unwrap()
            .text()
            .await
            .ok()
            .unwrap();
        let doc = Document::from(html.as_str());
        let tag = match self.lib.get_type(&url) {
            0 => {
                WebPage::new(html).text().await;
                "article"
            }
            1 => "div",
            _ => "img",
        };
        let tag_count = doc
            .select(Name(tag))
            .filter_map(|n| n.attr("src"))
            .collect::<Vec<_>>()
            .len();
        print!("{}", tag_count);
        // let _links: Vec<_> = doc
        //     .select(Name("img"))
        //     .filter_map(|n| n.attr("src"))
        //     // .map(|a| Url::parse(a).unwrap().path().to_string())
        //     .collect();
    }

    pub async fn fetch<'a>(
        &self,
        _url: &'a str,
    ) {
    }

    pub async fn save(&self) {
        let file = File::with_options()
            .write(true)
            .create(true)
            .open(&self.downloader)
            .unwrap();
        serde_json::to_writer(&file, &serde_json::to_string(&self).unwrap())
            .unwrap();
    }

    pub async fn load(&mut self) {
        let reader = BufReader::new(File::open(&self.downloader).unwrap());
        let contents: String = serde_json::from_reader(reader)
            .expect("The json has most likely been corrupted.");
        let Self {
            headers: h,
            sites: s,
            ..
        } = serde_json::from_str(&contents).unwrap();
        (self.headers, self.sites) = (h, s);
    }

    pub fn add_group_to_site(
        &mut self,
        site: String,
        group: &u32,
    ) {
        let v = self.sites.entry(site).or_default();
        if !v.contains(group) {
            v.push(*group);
        }
    }

    pub fn remove_group_to_site(
        &mut self,
        site: String,
        group: &u32,
    ) {
        let v = self.sites.entry(site).or_default();
        v.retain(|x| x != group);
    }

    pub fn add_header(
        &mut self,
        header: HeaderName,
        value: String,
        group: u32,
    ) {
        self.headers
            .entry(group)
            .or_default()
            .headers
            .insert(header, value.parse().unwrap());
    }

    pub fn remove_header(
        &mut self,
        header: HeaderName,
        group: u32,
    ) {
        if let Entry::Occupied(mut e) = self.headers.entry(group) {
            e.get_mut().headers.remove(header);
        }
    }

    pub fn remove_group(
        &mut self,
        group: &u32,
    ) {
        self.headers.remove(group);
    }
}

async fn _save_image(
    client: Client,
    url: Url,
    headers: HeaderMap,
    file: &mut File,
) -> Result<usize, std::io::Error> {
    file.write(
        &client
            .get(url.to_string())
            .headers(headers)
            .send()
            .await
            .ok()
            .unwrap()
            .bytes()
            .await
            .ok()
            .unwrap(),
    )
}
