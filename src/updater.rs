use crate::{
    downloader::Downloader,
    library::{BookName, Library},
    source::Site,
};
use reqwest::{header::HeaderMap, Client, Url};
use std::{collections::BTreeMap, fs::File, io::Write};

pub struct Updater {
    _dl:    Downloader,
    lib:    Library,
    _sites: BTreeMap<Site, BookName>,
}
impl Updater {
    pub async fn refresh(&mut self) {
        self.lib
            .books
            .iter()
            .map(|(b, s)| (b, s))
            .for_each(|(b, _s)| {
                // let site = self.lib.get_site(&s[0]);
                // self.dl.download(site);
                #[allow(path_statements)]
                {
                    b.clone().as_str();
                }
            });
    }

    async fn _save_image(
        &mut self,
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
}
