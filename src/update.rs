use crate::{
    library::{Book, BookName, Chapter, Library},
    retriever::Retriever,
    source::{Site, Source},
};
use futures::future::join_all;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Default, Clone, Debug)]
pub struct Manager {
    dl:       Retriever,
    lib:      Library,
    sites:    BTreeMap<BookName, Site>,
    updating: BTreeMap<BookName, Site>,
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

    pub async fn refresh(&mut self) -> u32 {
        let tmp = self.lib.books.clone();
        let iter = tmp
            .iter()
            .filter(|(name, _)| self.updating.contains_key(&*name))
            .map(|(a, b)| (a.clone(), b.clone()));
        let newchps = join_all(iter.map(|(a, b)| async {
            let (name, book) = (a, b);
            let pred = self.clone().delay(&name).await;
            let mut sources = vec![];
            let mut source = book.index.next(&pred).await;
            while let Some(src) = source {
                self.clone().delay(&name).await;
                if sources.contains(&src) {
                    break;
                };
                sources.push(src.clone());
                source = src.next(&pred).await;
            }
            let iter2 = sources.iter().cloned().map(|a| a.clone());
            join_all(iter2.map(|next| async {
                let mut book = book.clone();
                let ch = self.dl.clone().chapter(next, book.visual()).await;
                book.add_chapter(ch).await;
            }))
            .await
        }))
        .await;
        newchps.iter().map(|a| a).fold(0u32, |mut a, _b| {
            a += 1;
            a
        })
    }

    pub async fn delay(
        &mut self,
        name: &BookName,
    ) -> String {
        let mut sites = self.sites.clone();
        let site = sites.get_mut(name).unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        site.next.clone().to_string()
    }
}
