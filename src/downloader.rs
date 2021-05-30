use crate::library::Library;
use reqwest::{
    header::{HeaderMap, REFERER},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::BufReader};

impl Default for Downloader {
    fn default() -> Self {
        let mut h = BTreeMap::new();
        let mut s = BTreeMap::new();
        s.insert("manganato".to_string(), vec![0]);
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

impl Downloader {
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
}
