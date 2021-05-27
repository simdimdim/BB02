#![feature(with_options)]
#![feature(destructuring_assignment)]

use std::path::PathBuf;

pub mod downloader;

#[cfg(test)]
mod tests {
    use crate::downloader::Downloader;

    #[tokio::test]
    async fn it_works() {
        let mut dl = Downloader::default();
        dl.load().await;
        dl.save().await;
        drop(dl);
    }
}

#[derive(Debug, Clone)]
pub struct Book {
    visual: bool, //true = manga, false - text
    chapters: Vec<Content>,
}
#[derive(Debug, Clone)]
pub struct Chapter {
    n: u16,
    contents: Vec<Content>,
}
#[derive(Debug, Clone)]
pub struct Content(u8, PathBuf);
