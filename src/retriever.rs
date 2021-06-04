use crate::{
    library::{Chapter, Content},
    source::Source,
    CACHE,
};
use core::slice::SlicePattern;
use futures::future::join_all;
use reqwest::{
    header::{HeaderMap, REFERER},
    Client,
    Url,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    ops::Deref,
    path::PathBuf,
};

impl Default for Retriever {
    fn default() -> Self {
        let mut h = BTreeMap::new();
        let m = "readmanganato.com".to_string();
        h.insert(m.clone(), Headers::default());
        h.get_mut(&m)
            .unwrap()
            .headers
            .insert(REFERER, "https://readmanganato.com/".parse().unwrap());
        Self {
            client:   Client::new(),
            headers:  h,
            location: "/tmp/retriever.json".to_string(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retriever {
    headers:  BTreeMap<String, Headers>,
    #[serde(skip)]
    client:   Client,
    #[serde(skip)]
    location: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Headers {
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
}

impl Retriever {
    pub async fn fetch(
        &self,
        url: String,
    ) -> Source {
        Source::from(url).refresh().await
    }

    pub async fn chapter(
        &self,
        src: Source,
        visual: Option<bool>,
    ) -> Chapter {
        let mut ch = Chapter::default();
        // TODO: to be investigated
        ch.pos = src.place.0;
        let vis = visual.unwrap_or(src.check_visual().await.unwrap());
        let path = &PathBuf::from(CACHE).join(&src.title().deref());
        match vis {
            true => {
                join_all(
                    src.images_batch()
                        .await
                        .unwrap_or_default()
                        .iter()
                        .map(|s| self.content(s, true, path)),
                )
                .await
                .iter()
                .cloned()
                .for_each(|mut content| {
                    content.0 = src.place.1;
                    ch.add_content(content);
                });
            }
            false => {
                let cnt = self.content(&src.location, false, path).await;
                ch.add_content(cnt);
            }
        };
        ch.page = src;
        ch
    }

    pub async fn content(
        &self,
        source: &String,
        visual: bool,
        path: &PathBuf,
    ) -> Content {
        let src: Source = self.fetch(source.to_string()).await;
        let mut cnt = Content::default();
        cnt.0 = src.place.0;
        cnt.1 = cnt.1.join(path).join(&src.place.1.to_string());
        match visual {
            true => {
                cnt.save(
                    &self
                        .client
                        .get(source)
                        .headers(self.get_headers(source))
                        .send()
                        .await
                        .ok()
                        .unwrap()
                        .bytes()
                        .await
                        .ok()
                        .unwrap()
                        .as_slice(),
                );
            }
            false => {
                println!("{:?}", &src.text().await);
                let text = src.text().await.unwrap_or_default().join("\n\n");
                cnt.save(text.as_bytes());
            }
        }
        cnt
    }

    pub async fn save(&self) {
        let file = File::with_options()
            .write(true)
            .create(true)
            .open(&self.location)
            .unwrap();
        serde_json::to_writer(&file, &serde_json::to_string(&self).unwrap())
            .unwrap();
    }

    pub async fn load(&mut self) {
        let reader = BufReader::new(File::open(&self.location).unwrap());
        let contents: String = serde_json::from_reader(reader)
            .expect("The json has most likely been corrupted.");
        let Self { headers: h, .. } = serde_json::from_str(&contents).unwrap();
        self.headers = h;
    }

    fn get_headers(
        &self,
        src: &String,
    ) -> HeaderMap {
        self.headers
            .get(src.parse::<Url>().unwrap().domain().unwrap())
            .cloned()
            .unwrap_or_default()
            .headers
    }
}
