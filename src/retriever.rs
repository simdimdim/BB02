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
use std::{collections::BTreeMap, fs::File, io::BufReader};

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
                    content.0 = src.place.0;
                    content.1 = content.1.join(src.place.1.to_string());
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
        let src: Source = self.fetch(s.to_string()).await;
        let mut cnt = Content::default();
        cnt.0 = src.place.0;
        cnt.1 = cnt.1.join(src.place.1.to_string());
        match visual {
            true => {
                cnt.save(
                    &self
                        .client
                        .get(s)
                        .headers(self.get_headers(s))
                        .send()
                        .await
                        .ok()
                        .unwrap()
                        .bytes()
                        .await
                        .ok()
                        .unwrap(),
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
