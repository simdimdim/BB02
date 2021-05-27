#![feature(with_options)]
#![feature(destructuring_assignment)]

pub mod downloader;
pub mod library;

pub static TEST: &str="https://comrademao.com/mtl/i-was-trapped-on-the-same-day-for-100000-years/the-time-of-rebirth-chapter-1/";

#[cfg(test)]
mod tests {
    use crate::{downloader::Downloader, TEST};

    #[tokio::test]
    async fn download() {
        let mut dl = Downloader::default();
        dl.download(TEST.parse().unwrap()).await
    }
    #[tokio::test]
    async fn save() {
        let dl = Downloader::default();
        dl.save().await;
    }
    #[tokio::test]
    async fn load() {
        let mut dl = Downloader::default();
        dl.load().await;
    }
}
