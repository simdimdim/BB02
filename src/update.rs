use crate::{
    library::{Book, BookName, Chapter, Library},
    retriever::Retriever,
    source::{SiteInfo, Source},
};
use futures::future::join_all;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default, Clone, Debug)]
pub struct Manager {
    dl:    Retriever,
    lib:   Library,
    sites: Arc<Mutex<BTreeMap<String, SiteInfo>>>,
}
impl Manager {
    pub async fn add_book(
        &mut self,
        bookname: Option<BookName>,
        src: Source,
    ) {
        let source = src.refresh().await;
        let t = bookname.clone().unwrap_or(source.title());
        self.sites
            .lock()
            .await
            .insert(t.clone().to_string(), SiteInfo::new(&source));
        let mut book = Book::default();
        (book.name, book.index, book.pos) = (
            bookname.unwrap_or(source.title()),
            source.index().await,
            source.pos().await,
        );
        book.set_visual(None);
        let book = Arc::new(Mutex::new(book));

        let chapters: Vec<Chapter> = join_all(
            source
                .chapters()
                .await
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|url| async {
                    let bn = Source::new(url).await;
                    let timerkey = bn.domain();
                    self.sites.lock().await.entry(timerkey.clone()).or_default();
                    self.clone().delay(&timerkey).await;
                    self.dl.chapter(bn, None).await
                }),
        )
        .await;
        // join_all(chapters.iter_mut().map(|ch| ch.add_content(content))).await;
        for ch in chapters.iter().cloned() {
            let mut book = book.lock().await;
            book.add_chapter(ch).await;
        }
    }

    pub async fn refresh(&mut self) -> u32 {
        let tmp = self.lib.books.clone();
        let filter = true; // self.sites.lock().await.contains_key(&*name); //TODO: filter for some books
        let iter = tmp
            .iter()
            .filter(|(_name, _)| filter)
            .map(|(a, b)| (a.clone(), b.clone()));
        let newchps = join_all(iter.map(|(a, b)| async {
            let (name, book) = (a, b);
            let pred = self
                .clone()
                .delay(&name)
                .await
                .unwrap_or("Next".to_string());
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
                self.clone().delay(&name).await;
                let ch = self.dl.clone().chapter(next, book.visual()).await;
                book.add_chapter(ch).await
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
        name: &String,
    ) -> Option<String> {
        let mut site = self.sites.lock().await;
        dbg!(&site);
        let s = site.get_mut(name).unwrap();
        s.delay().await;
        s.next.clone()
    }
}
