use crate::{
    library::{Book, BookName, Library},
    retriever::Retriever,
    source::{SiteInfo, Source},
};
use futures::future::join_all;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Default, Clone, Debug)]
pub struct Manager {
    dl:    Retriever,
    lib:   Library,
    sites: Arc<Mutex<BTreeMap<String, SiteInfo>>>,
    preds: HashMap<String, String>,
}
impl Manager {
    pub async fn add_book(&mut self, bookname: Option<BookName>, source: Source) {
        let mut src = source.refresh().await;
        let bn = bookname.clone().unwrap_or(src.title()).to_string();
        self.sites.lock().await.insert(bn.clone(), SiteInfo::new());
        src = src.index().await.refresh().await;
        let mut book = Book::default();
        (book.name, book.index, book.pos) = (bn.into(), src.clone(), src.pos());
        book.set_visual(None);
        let book = Arc::new(Mutex::new(book));
        for ch in join_all(
            src.chapters()
                .await
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|url| async {
                    let bs = Source::new(url).await;
                    let domain = bs.domain();
                    self.sites
                        .lock()
                        .await
                        .entry(domain.clone())
                        .or_default()
                        .delay()
                        .await;
                    self.dl.chapter(bs, None).await
                }),
        )
        .await
        .iter()
        .cloned()
        {
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
            let pred = self.pred(&book.index);
            let mut sources = vec![];
            let mut source = book.index.next(&pred).await;
            while let Some(src) = source {
                if sources.contains(&src) {
                    break;
                };
                sources.push(src.clone());
                self.clone().delay(&name).await;
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

    pub async fn delay(&mut self, name: &String) -> Option<String> {
        let mut site = self.sites.lock().await;
        let s = site.get_mut(name).unwrap();
        s.delay().await;
        s.next.clone()
    }

    pub fn pred(&self, source: &Source) -> String {
        let default = "Next";
        self.preds
            .get(&source.domain())
            .cloned()
            .unwrap_or(default.to_string())
    }
}
