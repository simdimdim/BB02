use reqwest::{
    header::{HeaderMap, HeaderName, REFERER},
    Client, Url,
};
use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap};
use std::{fs::File, io::BufReader};

impl Default for Downloader {
    fn default() -> Self {
        let mut h = BTreeMap::new();
        let mut s = BTreeMap::new();
        s.insert("manganelo".to_string(), vec![0]);
        h.insert(0, Headers::default());
        Self {
            client: Client::new(),
            headers: h,
            sites: s,
            downloader: "/tmp/downloader_headers.json".to_string(),
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
    headers: BTreeMap<u32, Headers>,
    sites: BTreeMap<String, Vec<u32>>,
    #[serde(skip)]
    client: Client,
    downloader: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Headers {
    #[serde(with = "http_serde::header_map")]
    headers: HeaderMap,
}
impl Downloader {
    pub async fn download(&self, _url: Url) {}
    pub async fn fetch<'a>(&self, _url: &'a str) {}
    pub async fn save(&self) {
        let file = File::with_options()
            .write(true)
            .create(true)
            .open(&self.downloader)
            .unwrap();
        serde_json::to_writer(&file, &serde_json::to_string(&self).unwrap()).unwrap();
    }
    pub async fn load(&mut self) {
        let reader = BufReader::new(File::open(&self.downloader).unwrap());
        let contents: String = serde_json::from_reader(reader).unwrap();
        let Self {
            headers: h,
            sites: s,
            ..
        } = serde_json::from_str(&contents).unwrap();
        (self.headers, self.sites) = (h, s);
    }
    pub fn add_group_to_site(&mut self, site: String, group: &u32) {
        let v = self.sites.entry(site).or_default();
        if !v.contains(group) {
            v.push(*group);
        }
    }
    pub fn remove_group_to_site(&mut self, site: String, group: &u32) {
        let v = self.sites.entry(site).or_default();
        v.retain(|x| x != group);
    }
    pub fn add_header(&mut self, header: HeaderName, value: String, group: u32) {
        self.headers
            .entry(group)
            .or_default()
            .headers
            .insert(header, value.parse().unwrap());
    }
    pub fn remove_header(&mut self, header: HeaderName, group: u32) {
        match self.headers.entry(group) {
            Entry::Occupied(mut e) => {
                e.get_mut().headers.remove(header);
            }
            _ => {}
        }
    }
    pub fn remove_group(&mut self, group: &u32) {
        self.headers.remove(group);
    }
}
