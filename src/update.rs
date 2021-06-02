use crate::{
    downloader::Downloader,
    library::{Book, BookName, Chapter, Library},
    source::{Site, Source},
};
use futures::future::join_all;
use reqwest::{header::HeaderMap, Client, Url};
use std::{collections::BTreeMap, fs::File, io::Write, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct Manager {
    dl:    Downloader,
    lib:   Library,
    sites: BTreeMap<Site, BookName>,
}
impl Manager {
    pub async fn add_book(
        &mut self,
        bookname: Option<BookName>,
        source: Source,
    ) {
        let mut book = Book::default();
        (book.name, book.index, book.pos) = (
            bookname.unwrap_or(source.title().await),
            source.index().await,
            source.pos().await,
        );
        book.set_visual(None).await;
        let book = Arc::new(Mutex::new(book));

        // for ch in source
        //     .chapters()
        //     .await
        //     .unwrap_or_default()
        //     .iter()
        //     .cloned()
        //     .map(|c| Into::<Source>::into(c))
        // {
        //     book.add_chapter(ch).await;
        // }

        let _ = join_all(
            source
                .chapters()
                .await
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|c| Into::<Source>::into(c))
                .map(|ch| async {
                    book.lock().await.add_chapter(ch).await;
                }),
        );
        let _chapters: Vec<Chapter>;
    }

    pub async fn refresh(&mut self) {
        self.lib
            .books
            .iter()
            .map(|(b, s)| (b, s))
            .for_each(|(_, b)| {
                // let site = self.lib.get_site(&s[0]);
                // self.dl.download(site);
                #[allow(path_statements)]
                {
                    &b.index;
                    &self.dl;
                    &self.sites;
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
