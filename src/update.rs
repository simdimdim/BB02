use crate::{
    library::{Book, BookName, Chapter, Library},
    retriever::Retriever,
    source::{Site, Source},
};
use futures::future::join_all;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct Manager {
    dl:    Retriever,
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
            bookname.unwrap_or(source.title()),
            source.index().await,
            source.pos().await,
        );
        book.set_visual(None).await;
        let book = Arc::new(Mutex::new(book));
        #[allow(unused_mut)]
        let mut chapters: Vec<Chapter> = join_all(
            source
                .chapters()
                .await
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|url| async { self.dl.chapter(url.into(), None).await }),
        )
        .await;
        // join_all(chapters.iter_mut().map(|ch| ch.add_content(content))).await;
        join_all(chapters.iter().cloned().map(|ch| async {
            let mut book = book.lock().await;
            book.add_chapter(ch).await;
        }))
        .await;
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
}
