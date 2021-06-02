use crate::{
    library::{Chapter, Content, Library},
    source::Source,
};
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
    io::{BufReader, Write},
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
            lib:      Library::default(),
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
pub struct Retriever {
    headers:  BTreeMap<String, Headers>,
    #[serde(skip)]
    client:   Client,
    #[serde(skip)]
    location: String,
    lib:      Library,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Headers {
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
}

impl Retriever {
    pub async fn fetch(
        &self,
        _url: String,
    ) {
    }

    pub async fn chapter(
        &self,
        src: Source,
        visual: Option<bool>,
    ) -> Chapter {
        let mut ch = Chapter::default();
        ch.pos = src.num(); // TODO: make sure this works
        let vis = visual.unwrap_or(src.check_visual().await.unwrap());
        match vis {
            true => {
                join_all(
                    src.images_batch()
                        .await
                        .unwrap_or_default()
                        .iter()
                        .map(|s| self.content(s, true)),
                )
                .await
                .iter()
                .cloned()
                .for_each(|mut content| {
                    content.1 = content.1.join(format!("{}", src.num()));
                    ch.add_content(content);
                });
            }
            false => {
                let cnt = self.content(&src.location, false).await;
                ch.add_content(cnt);
            }
        };
        ch.page = src;
        ch
    }

    pub async fn content(
        &self,
        s: &String,
        visual: bool,
    ) -> Content {
        let src: Source = s.into();
        let mut cnt = Content::default();
        cnt.0 = src.num().into();
        cnt.1 = cnt.1.join(format!("{}", src.num()));
        match visual {
            true => {
                cnt.file()
                    .write(
                        &self
                            .client
                            .get(s)
                            .headers(
                                self.headers
                                    .get(
                                        &s.parse::<Url>()
                                            .unwrap()
                                            .domain()
                                            .unwrap()
                                            .to_string(),
                                    )
                                    .unwrap_or(&Headers::default())
                                    .clone()
                                    .headers,
                            )
                            .send()
                            .await
                            .ok()
                            .unwrap()
                            .bytes()
                            .await
                            .ok()
                            .unwrap(),
                    )
                    .unwrap();
            }
            false => {
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
}
